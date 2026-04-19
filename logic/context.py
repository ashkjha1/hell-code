"""
HELL-CODE Context — Repo Map Generator (F4)
Builds a lightweight summary of the workspace:
  - File tree (ignoring noise directories)
  - Function/class signatures extracted via regex for common languages
Injected into the agent prompt so it understands the codebase structure
without reading every file verbatim.
"""
import os
import re


# ---------------------------------------------------------------------------
# Config
# ---------------------------------------------------------------------------

IGNORED_DIRS = {
    ".git", "__pycache__", "node_modules", ".venv-hellcode",
    "target", ".mypy_cache", ".pytest_cache", "dist", "build",
    ".hell-code", ".vscode", ".idea", "scratch",
}

# Maximum file size to scan for signatures (bytes)
MAX_SCAN_BYTES = 100_000

# Language → (extension list, signature regex)
# Each regex must have one named group `sig` that captures the signature line.
_LANG_PATTERNS: list[tuple[list[str], re.Pattern]] = [
    # Python: def/class definitions
    (
        [".py"],
        re.compile(r"^(?P<sig>\s*(?:async\s+)?def\s+\w+\s*\(.*?\)|class\s+\w+(?:\(.*?\))?)\s*:", re.MULTILINE),
    ),
    # Rust: fn / struct / enum / impl / trait
    (
        [".rs"],
        re.compile(
            r"^(?P<sig>(?:pub(?:\([\w:]+\))?\s+)?(?:async\s+)?(?:fn|struct|enum|impl|trait)\s+\w+[^\{;]*)",
            re.MULTILINE,
        ),
    ),
    # JavaScript / TypeScript: function / class / arrow-function const
    (
        [".js", ".ts", ".jsx", ".tsx"],
        re.compile(
            r"^(?P<sig>(?:export\s+)?(?:default\s+)?(?:async\s+)?(?:function\s+\w+|class\s+\w+|const\s+\w+\s*=\s*(?:async\s+)?\())",
            re.MULTILINE,
        ),
    ),
    # Go: func definitions
    (
        [".go"],
        re.compile(r"^(?P<sig>func\s+(?:\(\w+\s+\*?\w+\)\s+)?\w+\s*\([^)]*\)[^{]*)", re.MULTILINE),
    ),
]

def _ext_to_pattern(ext: str) -> re.Pattern | None:
    for exts, pat in _LANG_PATTERNS:
        if ext in exts:
            return pat
    return None


# ---------------------------------------------------------------------------
# File tree
# ---------------------------------------------------------------------------

def _build_tree(root: str, max_depth: int = 4) -> list[str]:
    lines: list[str] = []

    def _walk(path: str, depth: int, prefix: str) -> None:
        if depth > max_depth:
            return
        try:
            entries = sorted(os.scandir(path), key=lambda e: (not e.is_dir(), e.name.lower()))
        except PermissionError:
            return
        for i, entry in enumerate(entries):
            if entry.name in IGNORED_DIRS or entry.name.startswith("."):
                continue
            connector = "└── " if i == len(entries) - 1 else "├── "
            lines.append(f"{prefix}{connector}{entry.name}{'/' if entry.is_dir() else ''}")
            if entry.is_dir():
                extension = "    " if i == len(entries) - 1 else "│   "
                _walk(entry.path, depth + 1, prefix + extension)

    lines.append(f"{os.path.basename(root) or root}/")
    _walk(root, 1, "")
    return lines


# ---------------------------------------------------------------------------
# Signature extraction
# ---------------------------------------------------------------------------

def _extract_signatures(filepath: str) -> list[str]:
    ext = os.path.splitext(filepath)[1].lower()
    pattern = _ext_to_pattern(ext)
    if pattern is None:
        return []
    try:
        size = os.path.getsize(filepath)
        if size > MAX_SCAN_BYTES:
            return []
        with open(filepath, "r", encoding="utf-8", errors="replace") as f:
            content = f.read()
        sigs = []
        for m in pattern.finditer(content):
            sig = m.group("sig").strip()
            # Truncate very long signatures
            if len(sig) > 120:
                sig = sig[:120] + "…"
            sigs.append(sig)
        return sigs[:30]  # cap per-file
    except Exception:
        return []


# ---------------------------------------------------------------------------
# Public API
# ---------------------------------------------------------------------------

def build_repo_map(root: str = ".", max_tree_depth: int = 4) -> str:
    """
    Return a markdown string with:
      1. File tree of the workspace
      2. Per-file function/class signatures for supported languages
    """
    root = os.path.realpath(root)
    tree_lines = _build_tree(root, max_tree_depth)
    tree_section = "## 🗂️  File Tree\n```\n" + "\n".join(tree_lines) + "\n```"

    sig_sections: list[str] = []
    for dirpath, dirnames, filenames in os.walk(root):
        # Prune ignored dirs in-place
        dirnames[:] = [d for d in dirnames if d not in IGNORED_DIRS and not d.startswith(".")]
        for fname in sorted(filenames):
            fpath = os.path.join(dirpath, fname)
            sigs = _extract_signatures(fpath)
            if sigs:
                rel = os.path.relpath(fpath, root)
                sig_sections.append(f"### `{rel}`\n" + "\n".join(f"  - {s}" for s in sigs))

    sig_block = ""
    if sig_sections:
        sig_block = "\n\n## 🔍  Signatures\n\n" + "\n\n".join(sig_sections)

    return tree_section + sig_block

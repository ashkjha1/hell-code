"""
HELL-CODE Tools — Real Filesystem & Shell Toolbox
Provides discrete, safe, callable functions for the agentic tool loop.
"""
import os
import re
import json
import subprocess
import shutil


# ---------------------------------------------------------------------------
# Guardrails
# ---------------------------------------------------------------------------

def _safe_path(path: str) -> str:
    """Resolve path relative to CWD and block obvious traversal attempts."""
    resolved = os.path.realpath(os.path.join(os.getcwd(), path))
    cwd = os.path.realpath(os.getcwd())
    if not resolved.startswith(cwd):
        raise PermissionError(f"Path '{path}' escapes the workspace root.")
    return resolved


# ---------------------------------------------------------------------------
# G1: read_file
# ---------------------------------------------------------------------------

def read_file(path: str, start_line: int = None, end_line: int = None) -> str:
    """
    Read a file's content, optionally limited to a line range.
    Returns the raw text content or an error string.
    """
    try:
        safe = _safe_path(path)
        if not os.path.exists(safe):
            return f"✖ File not found: {path}"
        with open(safe, "r", encoding="utf-8", errors="replace") as f:
            lines = f.readlines()

        if start_line is not None or end_line is not None:
            sl = max(0, (start_line or 1) - 1)
            el = end_line if end_line else len(lines)
            lines = lines[sl:el]

        MAX_CHARS = 20_000
        content = "".join(lines)
        if len(content) > MAX_CHARS:
            content = content[:MAX_CHARS] + f"\n\n[...truncated — file exceeds {MAX_CHARS} chars]"
        return content if content else "(empty file)"
    except PermissionError as e:
        return f"✖ Security Error: {e}"
    except Exception as e:
        return f"✖ read_file failed for '{path}': {e}"


# ---------------------------------------------------------------------------
# G1: list_dir
# ---------------------------------------------------------------------------

def list_dir(path: str = ".", max_depth: int = 3) -> str:
    """
    Return a recursive tree of the given directory up to max_depth.
    Respects .gitignore patterns by checking common ignored dirs.
    """
    IGNORED_DIRS = {".git", "__pycache__", "node_modules", ".venv-hellcode",
                    "target", ".mypy_cache", ".pytest_cache", "dist", "build"}
    try:
        safe = _safe_path(path)
        if not os.path.isdir(safe):
            return f"✖ Not a directory: {path}"

        lines = []

        def _walk(current: str, depth: int, prefix: str):
            if depth > max_depth:
                return
            try:
                entries = sorted(os.scandir(current), key=lambda e: (not e.is_dir(), e.name.lower()))
            except PermissionError:
                return
            for i, entry in enumerate(entries):
                if entry.name in IGNORED_DIRS:
                    continue
                connector = "└── " if i == len(entries) - 1 else "├── "
                lines.append(f"{prefix}{connector}{entry.name}{'/' if entry.is_dir() else ''}")
                if entry.is_dir():
                    extension = "    " if i == len(entries) - 1 else "│   "
                    _walk(entry.path, depth + 1, prefix + extension)

        lines.append(f"{path}/")
        _walk(safe, 1, "")
        return "\n".join(lines)
    except Exception as e:
        return f"✖ list_dir failed for '{path}': {e}"


# ---------------------------------------------------------------------------
# G1: grep_file
# ---------------------------------------------------------------------------

def grep_file(pattern: str, path: str = ".", recursive: bool = True,
              case_insensitive: bool = False, max_results: int = 50) -> str:
    """
    Search for a regex/literal pattern in files.
    Returns matching lines with file path and line numbers.
    """
    try:
        safe = _safe_path(path)
        flags = re.IGNORECASE if case_insensitive else 0
        matches = []

        def _search_file(fpath: str):
            try:
                with open(fpath, "r", encoding="utf-8", errors="replace") as f:
                    for lineno, line in enumerate(f, 1):
                        if re.search(pattern, line, flags):
                            rel = os.path.relpath(fpath, os.getcwd())
                            matches.append(f"{rel}:{lineno}: {line.rstrip()}")
                            if len(matches) >= max_results:
                                return True  # signal stop
            except Exception:
                pass
            return False

        if os.path.isfile(safe):
            _search_file(safe)
        elif recursive:
            for root, dirs, files in os.walk(safe):
                dirs[:] = [d for d in dirs if d not in {".git", "__pycache__", "node_modules", "target", ".venv-hellcode"}]
                for fname in files:
                    if _search_file(os.path.join(root, fname)):
                        break
                if len(matches) >= max_results:
                    break
        else:
            for fname in os.listdir(safe):
                fpath = os.path.join(safe, fname)
                if os.path.isfile(fpath):
                    if _search_file(fpath):
                        break

        if not matches:
            return f"No matches found for pattern '{pattern}' in '{path}'"
        header = f"Found {len(matches)} match(es) for '{pattern}':\n"
        return header + "\n".join(matches)
    except Exception as e:
        return f"✖ grep_file failed: {e}"


# ---------------------------------------------------------------------------
# G1: run_shell
# ---------------------------------------------------------------------------

def run_shell(command: str, cwd: str = ".", timeout: int = 30) -> str:
    """
    Execute a shell command in the given directory and return combined stdout+stderr.
    Hard timeout prevents runaway processes.
    """
    BLOCKED_PATTERNS = ["rm -rf /", ":(){ :|:& };:", "dd if=/dev/zero"]
    for blocked in BLOCKED_PATTERNS:
        if blocked in command:
            return f"✖ Blocked dangerous command pattern: '{blocked}'"
    try:
        safe_cwd = _safe_path(cwd)
        result = subprocess.run(
            command,
            shell=True,
            cwd=safe_cwd,
            capture_output=True,
            text=True,
            timeout=timeout,
        )
        out = result.stdout.strip()
        err = result.stderr.strip()
        exit_code = result.returncode
        parts = []
        if out:
            parts.append(out)
        if err:
            parts.append(f"[stderr]\n{err}")
        combined = "\n".join(parts) if parts else "(no output)"
        status = "✔" if exit_code == 0 else f"✖ (exit {exit_code})"
        return f"{status} $ {command}\n{combined}"
    except subprocess.TimeoutExpired:
        return f"✖ Command timed out after {timeout}s: {command}"
    except PermissionError as e:
        return f"✖ Security Error: {e}"
    except Exception as e:
        return f"✖ run_shell failed: {e}"


# ---------------------------------------------------------------------------
# write_file  (kept + improved with directory auto-creation)
# ---------------------------------------------------------------------------

def write_file(path: str, content: str) -> str:
    """
    Safely writes content to a file, creating parent directories as needed.
    F6: Before overwriting an existing file, backs it up to .hell-code/undo_stack/
        and records the entry in .hell-code/undo_stack/manifest.json so /undo works.
    """
    import time
    try:
        safe = _safe_path(path)
        directory = os.path.dirname(safe)
        if directory:
            os.makedirs(directory, exist_ok=True)

        # F6: Backup existing file before overwriting
        if os.path.isfile(safe):
            undo_dir = os.path.join(".hell-code", "undo_stack")
            os.makedirs(undo_dir, exist_ok=True)

            # Build a timestamped backup filename that preserves the relative path structure
            rel_path = os.path.relpath(safe, os.getcwd())
            safe_rel = rel_path.replace(os.sep, "__")
            timestamp = int(time.time())
            backup_name = f"{timestamp}__{safe_rel}"
            backup_path = os.path.join(undo_dir, backup_name)

            shutil.copy2(safe, backup_path)

            # Append to manifest.json
            manifest_path = os.path.join(undo_dir, "manifest.json")
            try:
                with open(manifest_path, "r", encoding="utf-8") as mf:
                    manifest = json.load(mf)
            except (FileNotFoundError, json.JSONDecodeError):
                manifest = []

            manifest.append({
                "original": rel_path,
                "backup": backup_path,
                "timestamp": timestamp,
            })

            # Keep at most 50 entries
            if len(manifest) > 50:
                oldest = manifest.pop(0)
                try:
                    os.remove(oldest["backup"])
                except OSError:
                    pass

            with open(manifest_path, "w", encoding="utf-8") as mf:
                json.dump(manifest, mf, indent=2)

        with open(safe, "w", encoding="utf-8") as f:
            f.write(content)
        return f"✔ Successfully wrote to {path}"
    except PermissionError as e:
        return f"✖ Security Error: {e}"
    except Exception as e:
        return f"✖ Failed to write to {path}: {e}"



# ---------------------------------------------------------------------------
# Tool dispatcher — used by run_agent_loop
# ---------------------------------------------------------------------------

# F4: Lazy import to avoid circular deps — context.py lives alongside tools.py
def repo_map(path: str = ".", max_depth: int = 4) -> str:
    """Return the repo map (file tree + signatures) for the workspace."""
    try:
        import context as _ctx
        return _ctx.build_repo_map(root=path, max_tree_depth=max_depth)
    except Exception as e:
        return f"✖ repo_map failed: {e}"


TOOL_REGISTRY = {
    "read_file": read_file,
    "list_dir": list_dir,
    "grep_file": grep_file,
    "run_shell": run_shell,
    "write_file": write_file,
    "repo_map": repo_map,   # F4
}

def dispatch_tool(tool_name: str, args: dict) -> str:
    """Look up and call a tool by name with the given dict of kwargs."""
    fn = TOOL_REGISTRY.get(tool_name)
    if fn is None:
        available = ", ".join(TOOL_REGISTRY.keys())
        return f"✖ Unknown tool '{tool_name}'. Available: {available}"
    try:
        return fn(**args)
    except TypeError as e:
        return f"✖ Bad arguments for tool '{tool_name}': {e}"

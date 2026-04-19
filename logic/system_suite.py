"""
HELL-CODE System Suite — Doctor and ACP utilities.
G3: All commands now load+append to the session for persistent memory.
G6 (partial): Doctor performs real environment validation.
"""
import os
import sys
import subprocess
import importlib
import llm_wrapper
import session_manager


# ─────────────────────────────────────────────────────────────────────────────
# G3 + G6(partial): /doctor — real environment checks with session memory
# ─────────────────────────────────────────────────────────────────────────────

def run_doctor(task, mode, model=None, api_key=None, api_base=None, session_id="default"):
    checks = []

    # 1. Check virtual environment
    venv_path = ".venv-hellcode"
    if os.path.isdir(venv_path):
        checks.append(f"✔ Python Environment: .venv-hellcode found at '{os.path.abspath(venv_path)}'")
    else:
        checks.append("✖ Python Environment: .venv-hellcode NOT found — run 'python -m venv .venv-hellcode && pip install -r requirements.txt'")

    # 2. Check litellm importable
    try:
        importlib.import_module("litellm")
        checks.append("✔ litellm: Importable")
    except ImportError:
        checks.append("✖ litellm: Not installed — run 'pip install litellm'")

    # 3. Check API key: existence + plausibility
    if api_key:
        prefix = api_key[:8] + "..."
        if len(api_key) >= 20:
            checks.append(f"✔ API Key: Detected (prefix: {prefix})")
        else:
            checks.append(f"⚠ API Key: Too short to be valid (prefix: {prefix})")
    else:
        checks.append("✖ API Key: Missing — add to .hell-code/secrets.toml")

    # 4. Check model name plausibility
    if model:
        checks.append(f"✔ Active Model: {model}")
    else:
        checks.append("⚠ Active Model: Not configured — check .hell-code/config.toml")

    # 5. Check git workspace
    try:
        branch = subprocess.check_output(
            ["git", "rev-parse", "--abbrev-ref", "HEAD"],
            stderr=subprocess.DEVNULL,
            text=True
        ).strip()
        dirty_raw = subprocess.check_output(
            ["git", "status", "--porcelain"],
            stderr=subprocess.DEVNULL,
            text=True
        ).strip()
        dirty_count = len(dirty_raw.splitlines()) if dirty_raw else 0
        checks.append(f"✔ Git: Branch '{branch}' — {dirty_count} modified file(s)")
    except Exception:
        checks.append("✖ Git: Not a git workspace or git not installed")

    # 6. Check skills directory
    skills_count = 0
    if os.path.isdir("skills"):
        skills_count = len([d for d in os.listdir("skills") if os.path.isdir(os.path.join("skills", d))])
    checks.append(f"✔ Skills: {skills_count} native skill(s) discovered")

    # 7. Check session storage
    session_dir = ".hell-code/sessions"
    if os.path.isdir(session_dir):
        session_files = [f for f in os.listdir(session_dir) if f.endswith(".json")]
        checks.append(f"✔ Sessions: {len(session_files)} session file(s) in {session_dir}")
    else:
        checks.append(f"⚠ Sessions: Directory '{session_dir}' not yet created")

    report = "## 🩺 HELL-CODE Doctor Report\n\n" + "\n".join(checks)

    # G3: Append to session so doctor results are part of conversation history
    session_manager.append_to_session(session_id, "user", "Run health check (/doctor)")
    session_manager.append_to_session(session_id, "assistant", report)

    return report


# ─────────────────────────────────────────────────────────────────────────────
# G3: /commit (ACP) — wired to session memory
# ─────────────────────────────────────────────────────────────────────────────

def run_acp(message, mode, model=None, api_key=None, api_base=None, session_id="default"):
    handler = llm_wrapper.LLMHandler(model_override=model, api_key_override=api_key, api_base_override=api_base)

    # G3: Load session history so AI has context when generating commit message
    history = session_manager.load_session(session_id)
    history_messages = [{"role": m["role"], "content": m["content"]} for m in history]

    # If no manual message, generate one from git diff
    if not message or message.strip() == "":
        try:
            diff = subprocess.check_output(["git", "diff", "--staged"], text=True)
            if not diff.strip():
                diff = subprocess.check_output(["git", "diff"], text=True)
            if not diff.strip():
                return "No changes to commit."
        except Exception as e:
            return f"✖ Could not read git diff: {e}"

        prompt = (
            f"Generate a concise, conventional commit message for these changes:\n\n{diff}\n\n"
            "Output ONLY the commit message line, no extra explanation."
        )
        message = handler.ask(prompt, system_prompt="You are a senior git engineer.", history=history_messages)

    # G3: Record the commit intent in the session
    session_manager.append_to_session(session_id, "user", f"Commit with message: {message}")

    try:
        subprocess.run(["git", "add", "."], check=True)
        subprocess.run(["git", "commit", "-m", message], check=True)
        subprocess.run(["git", "push"], check=True)
        result = f"✔ Successfully committed and pushed:\n> {message}"
    except subprocess.CalledProcessError as e:
        result = f"✖ ACP Failed: {e}"
    except Exception as e:
        result = f"✖ ACP Failed (unexpected): {e}"

    # G3: Append the result to the session
    session_manager.append_to_session(session_id, "assistant", result)
    return result

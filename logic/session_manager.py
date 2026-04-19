import os
import json
import uuid
import threading

SESSION_DIR = ".hell-code/sessions"
_session_lock = threading.Lock()

def get_session_file(session_id):
    if not os.path.exists(SESSION_DIR):
        os.makedirs(SESSION_DIR, exist_ok=True)
    return os.path.join(SESSION_DIR, f"{session_id}.json")

def load_session(session_id):
    path = get_session_file(session_id)
    if os.path.exists(path):
        with open(path, "r", encoding="utf-8") as f:
            try:
                return json.load(f)
            except json.JSONDecodeError:
                return []
    return []

def save_session(session_id, messages):
    path = get_session_file(session_id)
    with open(path, "w", encoding="utf-8") as f:
        json.dump(messages, f, indent=2)

def append_to_session(session_id, role, content):
    with _session_lock:
        messages = load_session(session_id)
        messages.append({"role": role, "content": content})
        save_session(session_id, messages)
        return messages

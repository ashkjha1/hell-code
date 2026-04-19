<div align="center">

```
██╗  ██╗███████╗██╗     ██╗      ██████╗ ██████╗ ██████╗ ███████╗
██║  ██║██╔════╝██║     ██║     ██╔════╝██╔═══██╗██╔══██╗██╔════╝
███████║█████╗  ██║     ██║     ██║     ██║   ██║██║  ██║█████╗  
██╔══██║██╔══╝  ██║     ██║     ██║     ██║   ██║██║  ██║██╔══╝  
██║  ██║███████╗███████╗███████╗╚██████╗╚██████╔╝██████╔╝███████╗
╚═╝  ╚═╝╚══════╝╚══════╝╚══════╝ ╚═════╝ ╚═════╝ ╚═════╝╚══════╝
```

**A production-grade, autonomous coding agent harness.**  
*Rust TUI shell. Python cognitive brain.*

![Rust](https://img.shields.io/badge/Rust-1.77%2B-orange?style=flat-square&logo=rust)
![Python](https://img.shields.io/badge/Python-3.11%2B-blue?style=flat-square&logo=python)
![LiteLLM](https://img.shields.io/badge/LiteLLM-multi--provider-purple?style=flat-square)
![License](https://img.shields.io/badge/license-MIT-green?style=flat-square)

</div>

---

## What is HELL-CODE?

HELL-CODE is a **hybrid terminal AI coding agent**: a [Ratatui](https://ratatui.rs)-powered Rust TUI for the interface, and a multi-agent Python orchestration brain for the cognition. It is designed to be a self-hosted, fully autonomous coding assistant that reads, writes, refactors, and commits code in your workspace — without leaving your terminal.

Inspired by [Claw Code](https://github.com/ultraworkers/claw-code) and [Claude Code](https://www.anthropic.com/claude-code), built for developers who want full control over their agent stack.

---

## ✨ Features

### 🧠 Autonomous Agent Loop
- **Tool-call iteration** — the agent plans, calls tools, observes results, and loops up to 8 rounds before delivering a final answer.
- **Real filesystem tools** — `read_file`, `write_file`, `list_dir`, `grep_file`, `run_shell`, `repo_map`.
- **`@[file]` mention injection** — reference any file in your prompt; its full content is injected automatically.

### 🗺️ Repo Map (F4)
- Generates a **file tree + function/class signatures** for the entire workspace.
- Supports Python, Rust, JavaScript/TypeScript, and Go out of the box.
- The agent can call `repo_map` as a tool to gain structural awareness without reading every file verbatim.

### ↩️ Undo Stack (F6)
- Every file the agent writes is **backed up** to `.hell-code/undo_stack/` with a JSON manifest.
- `/undo` restores the last written file — no accidental overwrites are permanent.

### 📟 Premium Catppuccin TUI
- **Catppuccin Mocha** colour palette throughout.
- Markdown rendering — bold, italics, inline code, fenced code blocks.
- Real-time thinking spinner, ghost-text autocomplete, scrollable conversation history.
- Floating help modal with dynamic slash command registry.

### 🔌 Multi-Provider LLM
- Powered by [LiteLLM](https://github.com/BerriAI/litellm) — swap between OpenAI, Anthropic, Gemini, OpenRouter, Ollama, and any OpenAI-compatible local server with a single config line.

### 💾 Persistent Sessions
- Full conversation history persisted as JSON in `.hell-code/sessions/`.
- Every slash command (`/doctor`, `/commit`, etc.) is session-aware — the agent remembers context across reboots.

---

## 📸 Interface Overview

```
┌─────────────────────────── HELL ▐████▌ CODE ────────────────────────────┐
│                                                                           │
│  🧠 HELL-CODE  (12:34:01)                                                │
│     Here are the changes I made to src/main.rs:                          │
│     **Added git-aware `/status`** — shells out to `git rev-parse`        │
│     and `git status --porcelain` for real branch + dirty count.          │
│                                                                           │
│  λ USER  (12:34:42)                                                       │
│     now add /undo support                                                 │
│                                                                           │
├───────────────────────────────────────────────────────────────────────────┤
│  [ EDITING ]  λ /undo▌                                                    │
├───────────────────────────────────────────────────────────────────────────┤
│ ▐ Execute ▌  claude-3.5-sonnet  ⠋ Thinking...  | Task: writing undo...   │
└───────────────────────────────────────────────────────────────────────────┘
```

---

## 🚀 Quick Start

### Prerequisites

| Dependency | Purpose |
|-----------|---------|
| Rust 1.77+ | Compiles the TUI shell |
| Python 3.11+ | Runs the cognitive brain |
| Git | Required for `/commit`, `/status`, `/diff` |

### Install & Run

```bash
# Clone
git clone https://github.com/ashkjha1/hell-code.git
cd hell-code

# Set up Python environment
python -m venv .venv-hellcode
source .venv-hellcode/bin/activate
pip install -r requirements.txt

# Configure (copy the examples and fill in your keys)
cp .hell-code/config.example.toml .hell-code/config.toml
cp .hell-code/secrets.example.toml .hell-code/secrets.toml

# Launch
cargo run
```

### Configure your LLM

**`.hell-code/config.toml`**
```toml
default_provider = "openrouter"

[openrouter]
active_model = "anthropic/claude-3.5-sonnet"

# Other supported providers:
# [openai]
# active_model = "gpt-4o"

# [anthropic]
# active_model = "claude-3-5-sonnet-20241022"

# [google_gemini]
# active_model = "gemini/gemini-2.5-flash"

# [local]
# active_model = "ollama/llama3.2"
# base_url = "http://localhost:11434"
```

**`.hell-code/secrets.toml`**
```toml
[api_keys]
openrouter = "sk-or-..."
openai     = "sk-..."
anthropic  = "sk-ant-..."
```

---

## ⌨️ Keybindings

| Key | Action |
|-----|--------|
| `i` | Enter Editing mode |
| `ESC` | Back to Normal mode / close modal |
| `TAB` | Toggle Plan ↔ Execute mode |
| `↑ / ↓` | Scroll conversation history |
| `→` | Accept ghost-text suggestion |
| `v` | Toggle verbose log panel |
| `?` | Toggle help modal |
| `q` | Quit |

---

## 🔧 Slash Commands

Type any command in Editing mode, prefixed with `/`:

| Command | Description |
|---------|-------------|
| `/doctor` | Check system health — venv, litellm, API key, git status |
| `/commit [msg]` | AI-powered Git Add-Commit-Push (generates message from diff if omitted) |
| `/init` | Initialize workspace; generate `HELL-CODE.md` with stack guidance |
| `/status` | Provider, model, git branch, dirty file count, work mode |
| `/diff` | `git diff --stat` |
| `/model [name]` | Switch active model live (takes effect on next message) |
| `/skills` | List loaded skill pipelines |
| `/agents` | List agent personas from `agents/` directory |
| `/clear` | Clear conversation history |
| `/undo` | Restore the last agent-written file from backup stack |
| `/help [cmd]` | Show help or detail a specific command |
| `/config` | Show config and secrets file paths |
| `/version` | Show HELL-CODE version |

---

## 🏗️ Architecture

```
hell-code/
├── src/                    # Rust TUI shell
│   ├── main.rs             # Event loop, slash command dispatch
│   ├── app.rs              # Application state
│   ├── ui.rs               # Ratatui rendering (chat, input, status, help)
│   ├── bridge.rs           # PyO3 ↔ Python bridge (call_agent, call_orchestrator)
│   ├── commands.rs         # SlashCommand enum + SLASH_COMMAND_SPECS registry
│   ├── theme.rs            # Catppuccin Mocha palette
│   ├── context.rs          # Project context injection (Rust side)
│   ├── config.rs           # TOML config + secrets loader
│   ├── logger.rs           # .hell-code/app.log writer
│   └── init.rs             # /init workspace initializer
│
├── logic/                  # Python cognitive engine
│   ├── orchestrator.py     # Zero-shot skill router
│   ├── dynamic_suite.py    # Agentic tool-call loop + @[file] injection
│   ├── tools.py            # Filesystem tools (read/write/grep/shell/repo_map)
│   ├── context.py          # Repo map generator (file tree + signatures)
│   ├── system_suite.py     # /doctor + /commit logic
│   ├── llm_wrapper.py      # LiteLLM multi-provider abstraction
│   ├── session_manager.py  # JSON session persistence
│   └── manager.py          # Background ThreadPoolExecutor task runner
│
├── skills/                 # Markdown skill pipelines
│   └── dev/SKILL.md        # Developer skill (lists agent pipeline)
│
├── agents/                 # Agent persona definitions (YAML frontmatter)
│   ├── main_agent.md
│   ├── junior_coder.md
│   ├── senior_reviewer.md
│   ├── legal_auditor.md
│   └── seo_analyst.md
│
└── .hell-code/             # Runtime data (gitignored)
    ├── config.toml         # Provider/model configuration
    ├── secrets.toml        # API keys
    ├── app.log             # Timestamped event log
    ├── sessions/           # Multi-turn conversation history (JSON)
    └── undo_stack/         # File backups + manifest.json for /undo
```

### How a prompt flows

```
User types a message
        │
        ▼
  starts_with('/') ?──Yes──▶ parse_slash_command() ──▶ handle_slash_command()
        │
        No
        ▼
  call_orchestrator() [Rust → PyO3 → Python]
        │
        ▼
  orchestrator.py — zero-shot LLM skill selection
        │
        ▼
  dynamic_suite.execute_skill()
    ├── inject_file_mentions() — @[file] expansion
    ├── load agent persona (agents/*.md)
    └── run_agent_loop()
          ├── LLM call
          ├── parse [TOOL_CALL] blocks
          ├── dispatch_tool() → read_file / run_shell / write_file / repo_map / …
          ├── feed result back as next user turn
          └── repeat (max 8 iterations)
        │
        ▼
  AgentEvent::Complete ──▶ Rust TUI renders response bubble
```

---

## 🧩 Extending HELL-CODE

### Add a custom skill

Create `skills/<name>/SKILL.md`:
```markdown
---
name: my-skill
description: Does something cool
---

- junior_coder
- senior_reviewer
```

The filename of each bullet must match a `.md` file in `agents/`.

### Add a custom agent

Create `agents/<name>.md` (or `.hell-code/agents/<name>.md` for per-project overrides):
```markdown
---
name: my-agent
role: specialist
---

You are a specialist in <domain>. When given a task, you ...
```

### Add a custom slash command

1. Add a variant to `SlashCommand` in `src/commands.rs`
2. Add a `CommandSpec` to `SLASH_COMMAND_SPECS`
3. Add a parse arm in `parse_slash_command()`
4. Add a handler arm in `handle_slash_command()` in `src/main.rs`

---

## ⚙️ Workspace Config (`.hell-code.json`)

Place `.hell-code.json` at the project root to set per-workspace permissions:

```json
{
  "permissions": {
    "defaultMode": "dontAsk"
  }
}
```

| Field | Values | Description |
|-------|--------|-------------|
| `permissions.defaultMode` | `"dontAsk"` | Agent writes files and runs shell commands without prompting _(default)_ |
| `permissions.defaultMode` | `"ask"` | Reserved for future diff-confirm flow |

---

## 🐛 Troubleshooting

| Problem | Solution |
|---------|----------|
| **`libpython3.12.dylib` not loaded** (macOS) | Verify `.cargo/config.toml` has both `@loader_path/../../` and `@loader_path/../../../` rpath entries |
| **LLM not responding** | Check `.hell-code/app.log` for `CRITICAL` entries; run `/doctor` |
| **Wrong model being used** | Use `/model <name>` — takes effect immediately on the next message |
| **Agent wrote a bad file** | Run `/undo` to restore from the backup stack |
| **Session feels stale** | Run `/clear` to reset conversation history |

---

## 🗺️ Roadmap

| Item | Status |
|------|--------|
| Real filesystem tool loop | ✅ Done |
| `@[file]` mention injection | ✅ Done |
| Repo map context generator | ✅ Done |
| `/undo` backup stack | ✅ Done |
| Git-aware `/status` | ✅ Done |
| Dynamic `/agents` listing | ✅ Done |
| **Diff preview + y/n confirm** | 🔲 Planned |
| **Streaming LLM response** | 🔲 Planned |

---

## 📄 License

MIT — see [LICENSE](LICENSE).

---

<div align="center">
Built with 🦀 Rust + 🐍 Python · Catppuccin Mocha · LiteLLM
</div>

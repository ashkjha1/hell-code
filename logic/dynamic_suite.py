"""
HELL-CODE Dynamic Suite — Skill execution with agentic tool-call loop.
G2: run_agent_loop() drives plan→tool→observe→tool→answer iterations.
F1: @[file] mention syntax is resolved before sending the prompt to the LLM.
"""
import os
import re
import json
import llm_wrapper
import session_manager
import tools

# ─────────────────────────────────────────────────────────────────────────────
# F1: @[file] mention resolution
# ─────────────────────────────────────────────────────────────────────────────

_MENTION_RE = re.compile(r"@\[([^\]]+)\]")

def inject_file_mentions(text: str) -> str:
    """
    Scan *text* for @[relative/path] patterns and replace each match with
    the full file contents wrapped in a labeled fence, so the LLM has
    direct access to the referenced code.
    """
    def _replace(match):
        path = match.group(1).strip()
        content = tools.read_file(path)
        ext = os.path.splitext(path)[1].lstrip(".")
        lang = ext if ext else ""
        return (
            f"\n\n--- Content of `{path}` ---\n"
            f"```{lang}\n{content}\n```\n"
            f"--- End of `{path}` ---\n"
        )
    return _MENTION_RE.sub(_replace, text)


# ─────────────────────────────────────────────────────────────────────────────
# G2: Tool-call tag parsing
# ─────────────────────────────────────────────────────────────────────────────

_TOOL_CALL_RE = re.compile(
    r"\[TOOL_CALL\]\s*(\{.*?\})\s*\[/TOOL_CALL\]",
    re.DOTALL
)
_WRITE_FILE_RE = re.compile(
    r"\[WRITE_FILE:\s*(.*?)\](.*?)\[/WRITE_FILE\]",
    re.DOTALL
)


def _extract_tool_calls(response: str) -> list[dict]:
    """Parse all [TOOL_CALL]{...}[/TOOL_CALL] blocks out of a response."""
    calls = []
    for match in _TOOL_CALL_RE.finditer(response):
        try:
            call = json.loads(match.group(1).strip())
            if "tool" in call:
                calls.append(call)
        except json.JSONDecodeError:
            pass
    return calls


def _extract_write_files(response: str) -> list[tuple[str, str]]:
    """Parse legacy [WRITE_FILE: path] ... [/WRITE_FILE] blocks."""
    return re.findall(_WRITE_FILE_RE, response)


def _strip_tool_calls(response: str) -> str:
    """Remove [TOOL_CALL] blocks from a response string so they aren't shown raw."""
    return _TOOL_CALL_RE.sub("", response).strip()


# ─────────────────────────────────────────────────────────────────────────────
# G2: build tool-use system prompt injection
# ─────────────────────────────────────────────────────────────────────────────

TOOL_INSTRUCTIONS = """
## Available Tools

You may invoke tools by embedding **exactly one** JSON block per call using this syntax:

[TOOL_CALL]
{"tool": "<name>", "args": {<key>: <value>, ...}}
[/TOOL_CALL]

Available tools:
- **read_file** — args: `path` (str), optional `start_line`/`end_line` (int)
- **list_dir**  — args: `path` (str, default "."), optional `max_depth` (int)
- **grep_file** — args: `pattern` (str), optional `path`, `recursive`, `case_insensitive`, `max_results`
- **run_shell** — args: `command` (str), optional `cwd`, `timeout`
- **write_file** — args: `path` (str), `content` (str)
- **repo_map**  — args: optional `path` (str, default "."), `max_depth` (int). Returns file tree + function signatures for the whole workspace.

Rules:
1. Use tools when you need real information from the filesystem or shell.
2. Emit tool calls first — wait for the result before continuing your answer.
3. After all tool calls are resolved, output your final answer with NO remaining [TOOL_CALL] blocks.
4. In PLANNING MODE do NOT emit write_file calls.
"""


# ─────────────────────────────────────────────────────────────────────────────
# G2: Agentic loop
# ─────────────────────────────────────────────────────────────────────────────

MAX_LOOP_ITERATIONS = 8  # safety cap


def run_agent_loop(
    handler: llm_wrapper.LLMHandler,
    system_prompt: str,
    initial_prompt: str,
    mode: str,
    history_messages: list[dict] | None = None,
) -> str:
    """
    Drive an autonomous tool-call loop:
      prompt → response → parse tool calls → execute → inject results → repeat
    until the agent gives a final answer with no pending tool calls.

    Returns the final text response (tool calls stripped).
    """
    messages = []
    if history_messages:
        messages.extend(history_messages)

    current_prompt = initial_prompt
    accumulated_tool_log = []

    for iteration in range(MAX_LOOP_ITERATIONS):
        response = handler.ask(current_prompt, system_prompt=system_prompt, history=messages)

        tool_calls = _extract_tool_calls(response)
        write_files = _extract_write_files(response) if mode == "execute" else []

        # Execute [WRITE_FILE] legacy tags
        for path, content in write_files:
            result = tools.write_file(path.strip(), content.strip())
            accumulated_tool_log.append(f"  write_file({path.strip()}) → {result}")

        # If no structured tool calls, we're done
        if not tool_calls:
            # Append final assistant turn to messages for multi-turn continuity
            messages.append({"role": "assistant", "content": response})
            break

        # Execute each tool call and collect results
        tool_results = []
        for call in tool_calls:
            tool_name = call.get("tool", "")
            tool_args = call.get("args", {})
            result = tools.dispatch_tool(tool_name, tool_args)
            snippet = result[:500] + "..." if len(result) > 500 else result
            tool_results.append(
                f"Tool: {tool_name}({json.dumps(tool_args)})\nResult:\n{snippet}"
            )
            accumulated_tool_log.append(f"  {tool_name}({json.dumps(tool_args)}) → {snippet[:120]}")

        # Feed results back as the next user turn
        clean_response = _strip_tool_calls(response)
        if clean_response:
            messages.append({"role": "assistant", "content": clean_response})

        current_prompt = "Tool Results:\n\n" + "\n\n---\n\n".join(tool_results) + "\n\nContinue your response based on these results."
        messages.append({"role": "user", "content": current_prompt})

    else:
        # Hit the iteration cap
        response = f"[Loop cap of {MAX_LOOP_ITERATIONS} iterations reached]\n\n" + response

    final = _strip_tool_calls(response)

    if accumulated_tool_log:
        tool_summary = "\n\n---\n**🔧 Tool Calls Executed:**\n" + "\n".join(accumulated_tool_log)
        final = final + tool_summary

    return final


# ─────────────────────────────────────────────────────────────────────────────
# Main entry point: execute_skill
# ─────────────────────────────────────────────────────────────────────────────

def execute_skill(skill_name, task, mode, model=None, api_key=None, api_base=None, session_id="default"):
    handler = llm_wrapper.LLMHandler(model_override=model, api_key_override=api_key, api_base_override=api_base)

    # F1: Resolve @[file] mentions in the user task before anything else
    task = inject_file_mentions(task)

    user_skill_path = os.path.join(".hell-code", "skills", skill_name, "SKILL.md")
    native_skill_path = os.path.join("skills", skill_name, "SKILL.md")
    skill_path = user_skill_path if os.path.exists(user_skill_path) else native_skill_path

    if not os.path.exists(skill_path):
        return f"Error: Skill '{skill_name}' not found at {user_skill_path} or {native_skill_path}"

    with open(skill_path, "r", encoding="utf-8") as f:
        skill_raw = f.read()

    skill_content = skill_raw
    if skill_raw.startswith("---"):
        try:
            _, _, skill_content = skill_raw.split("---", 2)
        except Exception:
            pass

    # Extract pipeline agents from bullet-point list
    agents = [
        line.strip()[2:].strip()
        for line in skill_content.split("\n")
        if line.strip().startswith("- ")
    ]

    if not agents:
        return f"Error: No agents defined in {skill_name}/SKILL.md (use bullet points '- agent_name')"

    # G3 (via caller): load session history for LLM multi-turn
    session_manager.append_to_session(session_id, "user", task)
    history = session_manager.load_session(session_id)
    history_messages = [{"role": m["role"], "content": m["content"]} for m in history[:-1]]

    full_response = ""

    for idx, agent in enumerate(agents):
        user_agent_file = os.path.join(".hell-code", "agents", f"{agent}.md")
        native_agent_file = os.path.join("agents", f"{agent}.md")
        agent_file = user_agent_file if os.path.exists(user_agent_file) else native_agent_file

        if not os.path.exists(agent_file):
            return f"Error: Agent persona '{agent}' not found"

        with open(agent_file, "r", encoding="utf-8") as f:
            agent_raw = f.read()

        system_prompt = agent_raw
        if agent_raw.startswith("---"):
            try:
                _, _, system_prompt = agent_raw.split("---", 2)
            except Exception:
                pass

        # Mode-specific instructions + tool-use instructions
        if mode == "execute":
            system_prompt = (
                "EXECUTION MODE: You are authorized to create or edit files. "
                "If the user doesn't specify a filename, YOU should decide on an appropriate name.\n"
                "To write to a file, use [WRITE_FILE: path]content[/WRITE_FILE] syntax,\n"
                "OR use the structured tool-call syntax below.\n\n"
                + TOOL_INSTRUCTIONS
                + "\n\n"
                + system_prompt.strip()
            )
        elif mode == "plan":
            system_prompt = (
                "PLANNING MODE: You are in an architecture phase. DO NOT write full code or emit write_file calls. "
                "Output a high-level strategy, file structure, and implementation roadmap.\n"
                "You MAY use read_file, list_dir, grep_file to understand the codebase.\n\n"
                + TOOL_INSTRUCTIONS
                + "\n\n"
                + system_prompt.strip()
            )

        prompt = (
            f"Session History (last few turns for context):\n"
            + "\n".join([f"{m['role'].upper()}: {m['content'][:300]}" for m in history_messages[-4:]])
            + f"\n\nTask for {agent}: {task}"
        )

        # G2: Run the agentic loop instead of a single LLM call
        response = run_agent_loop(
            handler=handler,
            system_prompt=system_prompt.strip(),
            initial_prompt=prompt,
            mode=mode,
            history_messages=history_messages,
        )

        section_title = f"### Phase {idx + 1}: {agent.replace('_', ' ').title()}"
        full_response += f"\n\n{section_title}\n{response}"

    session_manager.append_to_session(session_id, "assistant", full_response)
    return full_response

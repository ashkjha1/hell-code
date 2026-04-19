import json
import os
import llm_wrapper

def route_request(prompt, available_skills, model=None, api_key=None, api_base=None):
    handler = llm_wrapper.LLMHandler(model_override=model, api_key_override=api_key, api_base_override=api_base)
    
    # Load Main Agent persona
    persona_path = "agents/main_agent.md"
    if not os.path.exists(persona_path):
        # Fallback if file missing
        persona_content = "You are the HELL-CODE Main Agent. Decompose user requests into subtasks."
    else:
        with open(persona_path, "r", encoding="utf-8") as f:
            persona_content = f.read()
            # Strip YAML frontmatter if present
            if persona_content.startswith("---"):
                try:
                    _, _, persona_content = persona_content.split("---", 2)
                except:
                    pass

    system_prompt = persona_content.replace("{available_skills}", ", ".join(available_skills))
    user_input = f"User Request: {prompt}\n\nDecompose this into necessary skills and tasks. Available skills: {', '.join(available_skills)}"
    
    response = handler.ask(user_input, system_prompt=system_prompt.strip())
    
    try:
        clean_res = response.replace("```json", "").replace("```", "").strip()
        data = json.loads(clean_res)
        return data
    except Exception as e:
        # Fallback to single dev task if orchestration fails
        return {
            "thought_process": "Orchestration failed, falling back to basic dev skill.",
            "subtasks": [
                {"skill": "dev", "task": prompt, "background": False}
            ]
        }

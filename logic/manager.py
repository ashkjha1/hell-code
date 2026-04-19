import concurrent.futures
import dynamic_suite
import system_suite
import orchestrator

def run_dynamic(skill_name, task, mode, model=None, api_key=None, api_base=None, session_id="default"):
    if skill_name == "doctor":
        return system_suite.run_doctor(task, mode, model, api_key, api_base, session_id)
    if skill_name == "acp":
        return system_suite.run_acp(task, mode, model, api_key, api_base, session_id)
    # Default behavior for all other skills (dev, seo, legal, etc)
    return dynamic_suite.execute_skill(skill_name, task, mode, model, api_key, api_base, session_id)

def run_orchestrated(prompt, available_skills, mode, model=None, api_key=None, api_base=None, session_id="default"):
    # Phase 1: Orchestration / Thought Process
    plan = orchestrator.route_request(prompt, available_skills, model, api_key, api_base)
    
    thought_process = plan.get("thought_process", "Processing requirements...")
    subtasks = plan.get("subtasks", [])
    
    if not subtasks:
        # Fallback if no subtasks provided
        return f"### 🧠 Thought Process\n{thought_process}\n\nNo subtasks identified."

    results = []
    
    # We use a ThreadPoolExecutor for background execution
    with concurrent.futures.ThreadPoolExecutor() as executor:
        future_to_task = {}
        
        for task_info in subtasks:
            skill = task_info.get("skill", "dev")
            task_prompt = task_info.get("task", prompt)
            is_background = task_info.get("background", False)
            
            if is_background:
                # Submit to background
                future = executor.submit(run_dynamic, skill, task_prompt, mode, model, api_key, api_base, session_id)
                future_to_task[future] = f"Skill: {skill.upper()}"
            else:
                # Run synchronously (blocking)
                result = run_dynamic(skill, task_prompt, mode, model, api_key, api_base, session_id)
                results.append(f"## 🛠️ Task: {skill.upper()}\n{result}")

        # Collect background results
        for future in concurrent.futures.as_completed(future_to_task):
            task_label = future_to_task[future]
            try:
                result = future.result()
                results.append(f"## 🛠️ Background Task: {task_label}\n{result}")
            except Exception as exc:
                results.append(f"## 🛠️ Background Task: {task_label}\n✖ Error: {exc}")

    # Consolidate reporting
    final_report = f"# 🧠 Orchestration Report\n\n> {thought_process}\n\n"
    final_report += "\n\n---\n\n".join(results)
    
    return final_report
from llm_wrapper import LLMHandler

class JuniorCoder:
    def __init__(self, handler: LLMHandler):
        self.handler = handler

    def plan_implementation(self, task_with_context):
        prompt = f"""
        You are acting as a context-aware coding agent similar to Claude Code.
        Review the following task and repository context:
        {task_with_context}
        
        Provide a detailed implementation plan. Propose where new files should be created or which existing files should be modified.
        """
        return self.handler.ask(prompt, system_prompt="You are an Architect. Use the provided context to plan repository-wide changes.")

    def generate_code(self, task_with_context):
        prompt = f"""
        You are acting as a context-aware coding agent.
        Review the following task and repository context:
        {task_with_context}
        
        Generate the necessary code changes. If you are modifying existing files, specify the file path and the code block.
        If you are creating new files, provide the full content.
        """
        return self.handler.ask(prompt, system_prompt="You are a Senior Developer. Deliver precise code changes based on the project structure.")

class TestEngineer:
    def __init__(self, handler: LLMHandler):
        self.handler = handler

    def generate_tests(self, code_or_plan, is_plan=False):
        if is_plan:
            prompt = f"Design a test strategy for this plan:\n{code_or_plan}"
            sys_prompt = "You are a QA Lead."
        else:
            prompt = f"Write unit tests for the following implementation:\n{code_or_plan}"
            sys_prompt = "You are a Test Engineer."
        return self.handler.ask(prompt, system_prompt=sys_prompt)

class SeniorReviewer:
    def __init__(self, handler: LLMHandler):
        self.handler = handler

    def review(self, input_data, docs):
        prompt = f"Perform a final QA review on this implementation:\n{input_data}\nDocs:\n{docs}"
        return self.handler.ask(prompt, system_prompt="You are a Senior Reviewer.")

def execute_pipeline(task, mode, model=None, api_key=None, api_base=None):
    handler = LLMHandler(model_override=model, api_key_override=api_key, api_base_override=api_base)
    coder = JuniorCoder(handler)
    tester = TestEngineer(handler)
    reviewer = SeniorReviewer(handler)

    if mode == "plan":
        plan = coder.plan_implementation(task)
        test_strategy = tester.generate_tests(plan, is_plan=True)
        review = reviewer.review(plan, test_strategy)
        return f"--- REPO-AWARE PLAN ---\n{plan}\n\n--- TEST STRATEGY ---\n{test_strategy}\n\n--- FINAL REVIEW ---\n{review}"
    else:
        code = coder.generate_code(task)
        tests = tester.generate_tests(code, is_plan=False)
        review = reviewer.review(code, tests)
        return f"--- IMPLEMENTATION ---\n{code}\n\n--- UNIT TESTS ---\n{tests}\n\n--- QA VERIFICATION ---\n{review}"

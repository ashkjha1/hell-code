from llm_wrapper import LLMHandler

# Default "Golden Rules" library
GOLDEN_RULES = """
1. Liability Cap: Should not exceed the total value of the contract or $100,000, whichever is lower.
2. IP Ownership: All work product created during the engagement must belong to the Client.
3. Termination: Either party can terminate with 30-day written notice.
4. Confidentiality: Mutual non-disclosure agreement must be in place.
"""

def audit_contract(file_path, mode, model=None, api_key=None, api_base=None):
    handler = LLMHandler(model_override=model, api_key_override=api_key, api_base_override=api_base)
    
    try:
        with open(file_path, 'r') as f:
            contract_text = f.read()
    except Exception as e:
        return f"File Error: {str(e)}"
    
    context = "Initial Risk Assessment Strategy" if mode == "plan" else "Full Audit and Redlining"
    prompt = f"""
    Review the following contract against our 'Golden Rules' in {context} mode.
    
    GOLDEN RULES:
    {GOLDEN_RULES}
    
    CONTRACT TEXT:
    {contract_text}
    
    Provide:
    1. Compliance Status for each rule.
    2. Risk Identification (Non-standard clauses).
    3. Suggested Redlines (specific textual replacements).
    """
    
    audit = handler.ask(prompt, system_prompt="You are a professional Legal Auditor.")
    return audit

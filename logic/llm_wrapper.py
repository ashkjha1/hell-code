import os
import litellm
from litellm import completion
from dotenv import load_dotenv

load_dotenv()

class LLMHandler:
    """
    Unified handler for multiple LLM providers.
    Supports OpenAI, Gemini, Anthropic, OpenRouter, Ollama, LM Studio, etc.
    """
    def __init__(self, model_override=None, api_key_override=None, api_base_override=None):
        self.model = model_override or os.getenv("DEFAULT_MODEL", "gpt-4o-mini")
        
        # Auto-prefix gemini models for litellm AI Studio compatibility
        if "gemini" in self.model.lower() and not ("/" in self.model):
            self.model = f"gemini/{self.model}"
            
        self.api_base = api_base_override or os.getenv("CUSTOM_API_BASE")
        self.api_key = api_key_override or os.getenv("CUSTOM_API_KEY")

    def ask(self, prompt, system_prompt="You are HELL-CODE, a professional AI agent.", history=None):
        try:
            messages = [{"role": "system", "content": system_prompt}]
            if history:
                messages.extend(history)
            messages.append({"role": "user", "content": prompt})
            
            kwargs = {
                "model": self.model,
                "messages": messages,
            }
            if self.api_base:
                kwargs["api_base"] = self.api_base
            if self.api_key:
                kwargs["api_key"] = self.api_key

            response = completion(**kwargs)
            return response.choices[0].message.content
        except Exception as e:
            return f"Error calling LLM (Model: {self.model}): {str(e)}"

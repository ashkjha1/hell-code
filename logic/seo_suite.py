import requests
from bs4 import BeautifulSoup
from llm_wrapper import LLMHandler

def scrape_url(url):
    try:
        response = requests.get(url, timeout=10)
        soup = BeautifulSoup(response.text, 'html.parser')
        
        # Extract meaningful content
        h1s = [h.get_text() for h in soup.find_all('h1')]
        h2s = [h.get_text() for h in soup.find_all('h2')]
        meta_desc = soup.find('meta', attrs={'name': 'description'})
        meta_desc = meta_desc['content'] if meta_desc else "No description found"
        
        return {
            "h1": h1s,
            "h2": h2s,
            "description": meta_desc,
            "text_preview": soup.get_text()[:2000] # Limit for LLM context
        }
    except Exception as e:
        return {"error": str(e)}

def analyze_competitor(url, mode, model=None, api_key=None, api_base=None):
    handler = LLMHandler(model_override=model, api_key_override=api_key, api_base_override=api_base)
    data = scrape_url(url)
    
    if "error" in data:
        return f"Scraping Error: {data['error']}"
    
    context = "PLANNING" if mode == "plan" else "EXECUTION (Full Draft)"
    prompt = f"""
    Analyze the following competitor data for SEO gaps and content strategy in {context} mode.
    URL: {url}
    H1 Tags: {data['h1']}
    H2 Tags: {data['h2']}
    Description: {data['description']}
    Content Snippet: {data['text_preview']}
    
    Provide:
    1. Keyword Gap Analysis.
    2. Information Gain Opportunities.
    3. An optimized drafting structure (H-tag hierarchy).
    """
    
    analysis = handler.ask(prompt, system_prompt="You are an expert SEO Strategist.")
    return analysis

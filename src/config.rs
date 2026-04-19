use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use dialoguer::{Input, Select, theme::ColorfulTheme};
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ProviderConfig {
    pub models: Vec<String>,
    pub active_model: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct LocalConfig {
    pub base_url: String,
    pub models: Vec<String>,
    pub active_model: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct LegacyConfig {
    pub provider: String,
    pub model: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AppConfig {
    #[serde(default = "default_provider_name")]
    pub default_provider: String,
    pub openai: Option<ProviderConfig>,
    pub anthropic: Option<ProviderConfig>,
    pub google_gemini: Option<ProviderConfig>,
    pub openrouter: Option<ProviderConfig>,
    pub local: Option<LocalConfig>,
    // Legacy support
    pub llm: Option<LegacyConfig>,
}

fn default_provider_name() -> String { "openai".to_string() }

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Secrets {
    pub api_keys: HashMap<String, String>,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            default_provider: "openai".to_string(),
            openai: Some(ProviderConfig {
                models: vec!["gpt-4o".to_string(), "gpt-4o-mini".to_string()],
                active_model: "gpt-4o-mini".to_string(),
            }),
            anthropic: None,
            google_gemini: None,
            openrouter: None,
            local: None,
            llm: None,
        }
    }
}

fn get_config_dir() -> PathBuf { PathBuf::from(".hell-code") }
fn get_config_path() -> PathBuf { get_config_dir().join("config.toml") }
fn get_secrets_path() -> PathBuf { get_config_dir().join("secrets.toml") }

pub fn load_config_and_secrets(custom_path: Option<String>) -> (AppConfig, Option<Secrets>) {
    let config = load_config(custom_path);
    let secrets = read_secrets_file(&get_secrets_path()).ok();
    (config, secrets)
}

fn load_config(custom_path: Option<String>) -> AppConfig {
    if let Some(path) = custom_path {
        if let Ok(config) = read_config_file(&PathBuf::from(path)) { return config; }
    }
    if let Ok(config) = read_config_file(&PathBuf::from("hell_code.toml")) { return config; }
    if let Ok(config) = read_config_file(&get_config_path()) { return config; }
    AppConfig::default()
}

pub fn needs_onboarding() -> bool {
    !get_config_path().exists() || !Path::new(".venv-hellcode").exists()
}

fn setup_venv() -> Result<(), Box<dyn std::error::Error>> {
    println!("📦 Virtual environment not found. Creating '.venv-hellcode'...");
    let status = std::process::Command::new("python3").args(["-m", "venv", ".venv-hellcode"]).status()?;
    if !status.success() { return Err("Failed to create venv.".into()); }
    let pip_path = if cfg!(windows) { ".venv-hellcode\\Scripts\\pip" } else { ".venv-hellcode/bin/pip" };
    std::process::Command::new(pip_path).args(["install", "-r", "requirements.txt"]).status()?;
    Ok(())
}

pub fn run_onboarding() -> Result<(), Box<dyn std::error::Error>> {
    if !Path::new(".venv-hellcode").exists() { setup_venv()?; }
    if get_config_path().exists() { return Ok(()); }
    if !get_config_dir().exists() { fs::create_dir_all(get_config_dir())?; }

    println!("🚀 Welcome to HELL-CODE! Multiple Provider Setup Incoming.\n");

    let providers = vec!["OpenAI", "Anthropic", "Google Gemini", "OpenRouter", "Local (Ollama/LM Studio)"];
    let p_idx = Select::with_theme(&ColorfulTheme::default())
        .with_prompt("Select your PRIMARY provider")
        .items(&providers).default(0).interact()?;

    let provider_key = providers[p_idx].to_lowercase().replace(" ", "_").replace("local_(ollama/lm_studio)", "local");
    
    let model_map: HashMap<&str, Vec<&str>> = [
        ("openai", vec!["gpt-4o", "gpt-4o-mini"]),
        ("anthropic", vec!["claude-3-5-sonnet-latest", "claude-3-5-haiku-latest"]),
        ("google_gemini", vec!["gemini-2.5-pro", "gemini-2.5-flash"]),
        ("openrouter", vec!["openrouter/auto", "google/gemini-2.0-flash-001"]),
        ("local", vec!["llama3.1", "mistral", "phi3"]),
    ].iter().cloned().collect();

    let default_models = vec!["gpt-4o-mini"];
    let models = model_map.get(provider_key.as_str()).unwrap_or(&default_models);
    let m_idx = Select::with_theme(&ColorfulTheme::default())
        .with_prompt("Select the default model for this provider")
        .items(models).default(0).interact()?;

    let active_model = models[m_idx].to_string();
    let mut config = AppConfig {
        default_provider: provider_key.clone(),
        openai: None,
        anthropic: None,
        google_gemini: None,
        openrouter: None,
        local: None,
        llm: None,
    };

    let mut secrets = Secrets { api_keys: HashMap::new() };

    if provider_key == "local" {
        let base_url: String = Input::with_theme(&ColorfulTheme::default())
            .with_prompt("Local API Base URL").default("http://localhost:11434".to_string()).interact_text()?;
        config.local = Some(LocalConfig { base_url, models: vec![active_model.clone()], active_model });
    } else {
        let api_key: String = Input::with_theme(&ColorfulTheme::default())
            .with_prompt(format!("Enter your {} API Key", providers[p_idx])).interact_text()?;
        secrets.api_keys.insert(provider_key.clone(), api_key);
        
        let p_config = Some(ProviderConfig { models: vec![active_model.clone()], active_model: active_model.clone() });
        match provider_key.as_str() {
            "openai" => config.openai = p_config,
            "anthropic" => config.anthropic = p_config,
            "google_gemini" => config.google_gemini = p_config,
            "openrouter" => config.openrouter = p_config,
            _ => {}
        }
    }

    fs::write(get_config_path(), toml::to_string_pretty(&config)?)?;
    fs::write(get_secrets_path(), toml::to_string_pretty(&secrets)?)?;

    println!("\n✅ Configured for {}. You can add more providers manually in .hell-code/config.toml", provider_key);
    Ok(())
}

fn read_config_file(path: &Path) -> Result<AppConfig, Box<dyn std::error::Error>> {
    let content = fs::read_to_string(path)?;
    Ok(toml::from_str(&content)?)
}

fn read_secrets_file(path: &Path) -> Result<Secrets, Box<dyn std::error::Error>> {
    let content = fs::read_to_string(path)?;
    Ok(toml::from_str(&content)?)
}

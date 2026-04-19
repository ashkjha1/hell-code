use std::{error::Error, io, time::{Duration, Instant}};
use clap::{Parser, Subcommand};
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};
use tokio::sync::mpsc;

mod app;
mod bridge;
mod ui;
mod context;
mod config;
mod logger;
mod theme;
mod init;
mod commands;

pub use crate::app::{App, Mode, WorkMode, InputMode, MessageType};
use crate::bridge::{AgentEvent, call_agent, call_orchestrator};
use crate::config::{AppConfig, Secrets, load_config_and_secrets, needs_onboarding, run_onboarding};
use crate::logger::log_event;
use crate::commands::{SlashCommand, parse_slash_command};

#[derive(Parser, Clone)]
#[command(name = "hell-code")]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
    #[arg(short, long, global = true)]
    model: Option<String>,
    #[arg(short, long, global = true)]
    config: Option<String>,
}

#[derive(Subcommand, Clone, Debug)]
enum Commands {
    Doctor,
    Prompt { text: String },
    Acp { message: Option<String> },
    Auto { prompt: String },
    Init,
    #[command(external_subcommand)]
    Skill(Vec<String>),
}

use std::fs;
use std::collections::HashSet;

pub fn load_skills() -> Vec<String> {
    let mut skills_set = HashSet::new();
    
    // 1. Load native/default skills
    if let Ok(entries) = fs::read_dir("skills") {
        for entry in entries.flatten() {
            if entry.file_type().map(|t| t.is_dir()).unwrap_or(false) {
                if let Some(name) = entry.file_name().to_str() {
                    skills_set.insert(name.to_string());
                }
            }
        }
    }

    // 2. Load custom user-overrides
    if let Ok(entries) = fs::read_dir(".hell-code/skills") {
        for entry in entries.flatten() {
            if entry.file_type().map(|t| t.is_dir()).unwrap_or(false) {
                if let Some(name) = entry.file_name().to_str() {
                    skills_set.insert(name.to_string());
                }
            }
        }
    }
    
    let mut skills: Vec<String> = skills_set.into_iter().collect();
    skills.sort();
    skills
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let cli = Cli::parse();
    if needs_onboarding() { run_onboarding()?; }

    let (app_config, secrets) = load_config_and_secrets(cli.config.clone());
    log_event("INFO", "Application Started");

    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app = App::new();
    
    // Config Resolution with Legacy Support
    let (provider_name, model_name) = if let Some(ref legacy) = app_config.llm {
        (legacy.provider.clone(), legacy.model.clone())
    } else {
        let p = app_config.default_provider.clone();
        let m = cli.model.clone().unwrap_or_else(|| {
            match p.as_str() {
                "openai" => app_config.openai.as_ref().map(|c| c.active_model.clone()),
                "anthropic" => app_config.anthropic.as_ref().map(|c| c.active_model.clone()),
                "google_gemini" => app_config.google_gemini.as_ref().map(|c| c.active_model.clone()),
                "openrouter" => app_config.openrouter.as_ref().map(|c| c.active_model.clone()),
                "local" => app_config.local.as_ref().map(|c| c.active_model.clone()),
                _ => None,
            }.unwrap_or_else(|| "gpt-4o-mini".to_string())
        });
        (p, m)
    };
    
    app.set_config(provider_name.to_uppercase(), model_name.clone());
    app.available_skills = load_skills();
    let provider_key = provider_name.to_lowercase();

    let (tx, mut rx) = mpsc::unbounded_channel::<AgentEvent>();

    if let Some(ref cmd) = cli.command {
        start_task(&mut app, cmd.clone(), provider_key.clone(), model_name.clone(), &app_config, secrets.as_ref(), tx.clone());
    }

    let tick_rate = Duration::from_millis(50);
    let mut last_tick = Instant::now();

    loop {
        terminal.draw(|f| ui::render(f, &app))?;
        let timeout = tick_rate.checked_sub(last_tick.elapsed()).unwrap_or_else(|| Duration::from_secs(0));
        
        if event::poll(timeout)? {
            if let Event::Key(key) = event::read()? {
                match app.input_mode {
                    InputMode::Normal => match key.code {
                        KeyCode::Char('q') => app.quit(),
                        KeyCode::Char('i') => app.input_mode = InputMode::Editing,
                        KeyCode::Char('v') => app.verbose = !app.verbose,
                        KeyCode::Char('?') => app.show_help = !app.show_help,
                        KeyCode::Up => app.scroll_up(),
                        KeyCode::Down => app.scroll_down(),
                        KeyCode::Tab => app.toggle_work_mode(),
                        KeyCode::Enter => {
                            if let Some(ref cmd) = cli.command {
                                start_task(&mut app, cmd.clone(), provider_key.clone(), model_name.clone(), &app_config, secrets.as_ref(), tx.clone());
                            }
                        }
                        _ => {}
                    },
                    InputMode::Editing => match key.code {
                        KeyCode::Enter => {
                            let input = app.input.drain(..).collect::<String>();
                            app.input_mode = InputMode::Normal;
                            log_event("USER", &input);
                            app.add_message(MessageType::User, input.clone());

                            if input.starts_with('/') {
                                if let Some(slash_cmd) = parse_slash_command(&input) {
                                    // G4: read active_model at call-time, not from startup capture
                                    let current_model = app.active_model.clone();
                                    handle_slash_command(&mut app, slash_cmd, provider_key.clone(), current_model, &app_config, secrets.as_ref(), tx.clone());
                                } else {
                                    app.add_message(MessageType::System, format!("✖ Unknown command: {}", input));
                                }
                            } else {
                                // Default to Orchestrator (Auto mode) for non-prefixed input
                                // G4: read active_model at call-time
                                let current_model = app.active_model.clone();
                                let cmd = Commands::Auto { prompt: input.clone() };
                                start_task(&mut app, cmd, provider_key.clone(), current_model, &app_config, secrets.as_ref(), tx.clone());
                            }
                        }
                        KeyCode::Char(c) => {
                            app.input.push(c);
                            app.update_ghost_text();
                        }
                        KeyCode::Backspace => { 
                            app.input.pop(); 
                            app.update_ghost_text();
                        }
                        KeyCode::Right => {
                            if let Some(ghost) = app.ghost_text.take() {
                                app.input.push_str(&ghost);
                            }
                        }
                        KeyCode::Esc => app.input_mode = InputMode::Normal,
                        _ => {}
                    }
                }
            }
        }
        
        while let Ok(event) = rx.try_recv() {
            match event {
                AgentEvent::Log(msg) => { 
                    app.add_log(msg.clone()); 
                    log_event("AGENT", &msg); 
                }
                AgentEvent::Complete(res) => { 
                    app.add_message(MessageType::Assistant, res.clone()); 
                    app.add_log("Skill execution complete.".to_string()); 
                    log_event("SUCCESS", &format!("Result: {}", res));
                    app.set_status(Mode::Completed); 
                }
                AgentEvent::Error(err) => { 
                    app.add_message(MessageType::System, format!("✖ Error: {}", err)); 
                    log_event("CRITICAL", &err);
                    app.set_status(Mode::Error); 
                }
            }
        }
        if app.should_quit { break; }
        if last_tick.elapsed() >= tick_rate { 
            last_tick = Instant::now();
            app.tick_count += 1;
        }
    }
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen, DisableMouseCapture)?;
    terminal.show_cursor()?;
    log_event("INFO", "Application Shutdown");
    Ok(())
}

fn handle_slash_command(app: &mut App, cmd: SlashCommand, provider_key: String, model: String, config: &AppConfig, secrets: Option<&Secrets>, tx: mpsc::UnboundedSender<AgentEvent>) {
    match cmd {
        SlashCommand::Doctor => start_task(app, Commands::Doctor, provider_key, model, config, secrets, tx),
        SlashCommand::Init => start_task(app, Commands::Init, provider_key, model, config, secrets, tx),
        SlashCommand::Commit { message } => start_task(app, Commands::Acp { message }, provider_key, model, config, secrets, tx),
        SlashCommand::Clear => app.clear_messages(),
        SlashCommand::Help { command } => {
            if let Some(c) = command {
                if let Some(spec) = crate::commands::SLASH_COMMAND_SPECS.iter().find(|s| s.name == c || s.aliases.contains(&c.as_str())) {
                    app.add_message(MessageType::System, format!("Command: /{0}\nUsage: {1}\nCategory: {2}\nSummary: {3}", spec.name, spec.usage, spec.category, spec.summary));
                } else {
                    app.add_message(MessageType::System, format!("Unknown command: {}", c));
                }
            } else {
                app.show_help = true;
            }
        }
        SlashCommand::Status => {
            // G7: Expand status with git branch, dirty count, work mode
            let branch = std::process::Command::new("git")
                .args(["rev-parse", "--abbrev-ref", "HEAD"])
                .output()
                .ok()
                .and_then(|o| String::from_utf8(o.stdout).ok())
                .map(|s| s.trim().to_string())
                .unwrap_or_else(|| "(no git)".to_string());

            let dirty_count = std::process::Command::new("git")
                .args(["status", "--porcelain"])
                .output()
                .ok()
                .and_then(|o| String::from_utf8(o.stdout).ok())
                .map(|s| s.lines().count())
                .unwrap_or(0);

            let status = format!(
                "## ⚙️  HELL-CODE Status\n\
                 Provider  : {provider}\n\
                 Model     : {model}\n\
                 Mode      : {mode:?}\n\
                 Skills    : {skills}\n\
                 Git Branch: {branch}\n\
                 Dirty Files: {dirty}",
                provider = app.active_provider,
                model    = app.active_model,
                mode     = app.work_mode,
                skills   = app.available_skills.len(),
                branch   = branch,
                dirty    = dirty_count,
            );
            app.add_message(MessageType::System, status);
        }
        SlashCommand::Diff => {
            let output = std::process::Command::new("git")
                .args(["diff", "--stat"])
                .output();
            match output {
                Ok(out) => {
                    let diff = String::from_utf8_lossy(&out.stdout).to_string();
                    if diff.is_empty() {
                        app.add_message(MessageType::System, "No changes detected.".to_string());
                    } else {
                        app.add_message(MessageType::System, format!("Git Status (Stats):\n{}", diff));
                    }
                }
                Err(e) => app.add_message(MessageType::System, format!("✖ Failed to run git diff: {}", e)),
            }
        }
        SlashCommand::Skills => {
            let skills = app.available_skills.join(", ");
            app.add_message(MessageType::System, format!("Available Skills: {}", skills));
        }
        SlashCommand::Agents => {
            // G5: Dynamically scan agents/ and .hell-code/agents/ directories
            let mut agents_set = std::collections::HashSet::new();
            for dir in &["agents", ".hell-code/agents"] {
                if let Ok(entries) = fs::read_dir(dir) {
                    for entry in entries.flatten() {
                        let path = entry.path();
                        if path.extension().and_then(|e| e.to_str()) == Some("md") {
                            if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
                                agents_set.insert(stem.to_string());
                            }
                        }
                    }
                }
            }
            let mut agents: Vec<String> = agents_set.into_iter().collect();
            agents.sort();
            if agents.is_empty() {
                app.add_message(MessageType::System, "No agent personas found in agents/ directory.".to_string());
            } else {
                app.add_message(MessageType::System, format!("Available Agents ({}):\n  {}", agents.len(), agents.join("\n  ")));
            }
        }
        SlashCommand::Model { name } => {
            if let Some(n) = name {
                app.active_model = n.clone();
                // G4: Confirm that the switch is live — next LLM call will use this model
                app.add_message(MessageType::System, format!("✔ Model switched to: {}\n(Takes effect on your next message)", n));
            } else {
                app.add_message(MessageType::System, format!("Current Model: {}", app.active_model));
            }
        }
        SlashCommand::Version => {
            app.add_message(MessageType::System, format!("HELL-CODE v{}", env!("CARGO_PKG_VERSION")));
        }
        SlashCommand::Config => {
            let config_path = ".hell-code/config.toml";
            let secrets_path = ".hell-code/secrets.toml";
            let status = format!("Config Path: {0}\nSecrets Path: {1}\nProvider: {2}\nModel: {3}", config_path, secrets_path, app.active_provider, app.active_model);
            app.add_message(MessageType::System, status);
        }
        SlashCommand::Undo => {
            // F6: Restore last agent-written file from .hell-code/undo_stack/manifest.json
            let manifest_path = std::path::Path::new(".hell-code/undo_stack/manifest.json");
            let result = (|| -> Result<String, String> {
                if !manifest_path.exists() {
                    return Err("Undo stack is empty — no files have been written yet.".to_string());
                }
                let raw = fs::read_to_string(manifest_path)
                    .map_err(|e| format!("Failed to read undo manifest: {}", e))?;
                let mut entries: Vec<serde_json::Value> = serde_json::from_str(&raw)
                    .map_err(|e| format!("Corrupt undo manifest: {}", e))?;
                if entries.is_empty() {
                    return Err("Undo stack is empty.".to_string());
                }
                let last = entries.pop().unwrap();
                let original = last["original"].as_str()
                    .ok_or("Malformed entry: missing 'original'")?;
                let backup = last["backup"].as_str()
                    .ok_or("Malformed entry: missing 'backup'")?;
                let backup_path = std::path::Path::new(backup);
                if !backup_path.exists() {
                    return Err(format!("Backup file not found: {}", backup));
                }
                if let Some(parent) = std::path::Path::new(original).parent() {
                    fs::create_dir_all(parent)
                        .map_err(|e| format!("Failed to create parent dirs: {}", e))?;
                }
                fs::copy(backup_path, original)
                    .map_err(|e| format!("Failed to restore file: {}", e))?;
                // Save the trimmed manifest back
                let updated = serde_json::to_string_pretty(&entries)
                    .map_err(|e| format!("Failed to serialize manifest: {}", e))?;
                fs::write(manifest_path, updated)
                    .map_err(|e| format!("Failed to update manifest: {}", e))?;
                Ok(format!("✔ Restored '{}' from backup '{}'", original, backup))
            })();
            match result {
                Ok(msg) => app.add_message(MessageType::System, msg),
                Err(err) => app.add_message(MessageType::System, format!("✖ Undo failed: {}", err)),
            }
        }
    }
}

fn start_task(app: &mut App, cmd: Commands, provider_key: String, model: String, config: &AppConfig, secrets: Option<&Secrets>, tx: mpsc::UnboundedSender<AgentEvent>) {
    app.set_status(Mode::Processing);
    let work_mode = app.work_mode;
    let project_context = context::get_project_context("./");
    
    let api_base = if provider_key == "local" {
         config.local.as_ref().map(|c| c.base_url.clone())
    } else {
        None
    };

    let api_key = secrets.and_then(|s| s.api_keys.get(&provider_key).cloned());

    log_event("INFO", &format!("Invoking Skill: {:?} with model {}", cmd, model));

    match cmd {
        Commands::Doctor => {
            app.current_task = Some("Running Health Check...".to_string());
            tokio::spawn(call_agent("manager".to_string(), "run_dynamic".to_string(), "doctor".to_string(), "".to_string(), work_mode, Some(model), api_key, api_base, tx));
        }
        Commands::Prompt { text } => {
            app.current_task = Some(text.clone());
            tokio::spawn(call_agent("manager".to_string(), "run_dynamic".to_string(), "dev".to_string(), text, work_mode, Some(model), api_key, api_base, tx));
        }
        Commands::Acp { message } => {
            app.current_task = Some("Syncing changes...".to_string());
            let msg = message.unwrap_or_default();
            tokio::spawn(call_agent("manager".to_string(), "run_dynamic".to_string(), "acp".to_string(), msg, work_mode, Some(model), api_key, api_base, tx));
        }
        Commands::Auto { prompt } => {
            app.current_task = Some(prompt.clone());
            let skills = app.available_skills.clone();
            tokio::spawn(call_orchestrator("manager".to_string(), "run_orchestrated".to_string(), prompt, skills, work_mode, Some(model), api_key, api_base, tx));
        }
        Commands::Skill(args) => {
            if args.is_empty() { return; }
            let skill_name = args[0].clone();
            let task = if args.len() > 1 { args[1..].join(" ") } else { "".to_string() };
            
            app.current_task = Some(task.clone());
            let full_task = format!("{}\n\nCONTEXT:\n{}", task, project_context);
            
            tokio::spawn(call_agent("manager".to_string(), "run_dynamic".to_string(), skill_name, full_task, work_mode, Some(model), api_key, api_base, tx));
        }
        Commands::Init => {
            app.current_task = Some("Initializing workspace...".to_string());
            let tx_clone = tx.clone();
            tokio::spawn(async move {
                match init::initialize_repo(std::path::Path::new("./")) {
                    Ok(report) => {
                        let _ = tx_clone.send(AgentEvent::Log("Workspace initialized successfully.".to_string()));
                        let _ = tx_clone.send(AgentEvent::Complete(report.render()));
                    }
                    Err(e) => {
                        let _ = tx_clone.send(AgentEvent::Error(format!("Initialization failed: {}", e)));
                    }
                }
            });
        }
    }
}
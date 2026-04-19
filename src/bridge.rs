use pyo3::prelude::*;
use tokio::sync::mpsc;
use crate::app::WorkMode;

#[derive(Debug, Clone)]
pub enum AgentEvent {
    Log(String),
    Complete(String),
    Error(String),
}

pub async fn call_agent(
    module: String, 
    func: String, 
    skill_name: String,
    task: String, 
    work_mode: WorkMode, 
    model: Option<String>, 
    api_key: Option<String>,
    api_base: Option<String>,
    tx: mpsc::UnboundedSender<AgentEvent>
) {
    tokio::task::spawn_blocking(move || {
        Python::with_gil(|py| {
            let sys = py.import_bound("sys").expect("Failed to import sys");
            let path = sys.getattr("path").expect("Failed to get sys.path");
            path.call_method1("insert", (0, "./logic")).expect("Failed to insert logic directory");

            let py_mod = match py.import_bound(module.as_str()) {
                Ok(m) => m,
                Err(e) => { let _ = tx.send(AgentEvent::Error(format!("Import Error: {}", e))); return; }
            };
            
            let mode_str = match work_mode {
                WorkMode::Plan => "plan",
                WorkMode::Execute => "execute",
            };

            let _ = tx.send(AgentEvent::Log(format!("Agent initiated in {} mode...", mode_str)));
            
            // Call with (skill_name, task, mode, model, api_key, api_base)
            match py_mod.call_method1(func.as_str(), (skill_name, task, mode_str, model, api_key, api_base)) {
                Ok(res) => {
                    let output: String = res.extract().unwrap_or_else(|_| "Extraction Error".to_string());
                    let _ = tx.send(AgentEvent::Complete(output));
                }
                Err(e) => { let _ = tx.send(AgentEvent::Error(format!("Python Error: {}", e))); }
            }
        });
    }).await.expect("Task panicked");
}

pub async fn call_orchestrator(
    module: String,
    func: String,
    prompt: String,
    available_skills: Vec<String>,
    work_mode: WorkMode,
    model: Option<String>,
    api_key: Option<String>,
    api_base: Option<String>,
    tx: mpsc::UnboundedSender<AgentEvent>,
) {
    let mode_str = match work_mode {
        WorkMode::Plan => "plan",
        WorkMode::Execute => "execute",
    };

    tokio::task::spawn_blocking(move || {
        Python::with_gil(|py| {
            let sys = py.import_bound("sys").expect("Failed to import sys");
            let path = sys.getattr("path").expect("Failed to get sys.path");
            path.call_method1("insert", (0, "./logic")).expect("Failed to insert logic directory");

            let py_mod = match py.import_bound(module.as_str()) {
                Ok(m) => m,
                Err(e) => {
                    let _ = tx.send(AgentEvent::Error(format!("Module Import Error: {}", e)));
                    return;
                }
            };

            let _ = tx.send(AgentEvent::Log("Orchestrating multi-agent thought process...".to_string()));
            
            // Call with (prompt, available_skills, mode, model, api_key, api_base)
            match py_mod.call_method1(func.as_str(), (prompt, available_skills, mode_str, model, api_key, api_base)) {
                Ok(res) => {
                    let output: String = res.extract().unwrap_or_else(|_| "Extraction Error".to_string());
                    let _ = tx.send(AgentEvent::Complete(output));
                }
                Err(err) => {
                    let _ = tx.send(AgentEvent::Error(format!("Python Orchestration Error: {}", err)));
                }
            }
        });
    }).await.expect("Orchestrator Task panicked");
}
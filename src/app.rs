use std::collections::VecDeque;
use ratatui::widgets::ListState;

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum WorkMode {
    Plan,
    Execute,
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum InputMode {
    Normal,
    Editing,
}

#[derive(Debug, PartialEq)]
pub enum Mode {
    Idle,
    Processing,
    Completed,
    Error,
}

impl std::fmt::Display for Mode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Mode::Idle => write!(f, "Idle"),
            Mode::Processing => write!(f, "Processing"),
            Mode::Completed => write!(f, "Completed"),
            Mode::Error => write!(f, "Error"),
        }
    }
}

#[derive(Clone, Debug)]
pub enum MessageType {
    User,
    Assistant,
    System,
    Tool,
}

#[derive(Clone, Debug)]
pub struct Message {
    pub msg_type: MessageType,
    pub content: String,
    pub timestamp: String,
}

pub struct App {
    pub status_mode: String,
    pub work_mode: WorkMode,
    pub input_mode: InputMode,
    pub input: String,
    pub messages: Vec<Message>,
    pub logs: VecDeque<String>,
    pub current_task: Option<String>,
    pub should_quit: bool,
    // Configuration Context
    pub active_provider: String,
    pub active_model: String,
    // Dynamic Skills
    pub available_skills: Vec<String>,
    pub verbose: bool,
    pub show_help: bool,
    pub chat_state: ListState,
    pub log_state: ListState,
    pub tick_count: u64,
    pub ghost_text: Option<String>,
}

impl App {
    pub fn new() -> Self {
        Self {
            status_mode: "Idle".to_string(),
            work_mode: WorkMode::Plan,
            input_mode: InputMode::Normal,
            input: String::new(),
            messages: Vec::new(),
            logs: VecDeque::with_capacity(100),
            current_task: None,
            should_quit: false,
            active_provider: "Unknown".to_string(),
            active_model: "Unknown".to_string(),
            available_skills: Vec::new(),
            verbose: false,
            show_help: false,
            chat_state: ListState::default(),
            log_state: ListState::default(),
            tick_count: 0,
            ghost_text: None,
        }
    }

    pub fn set_config(&mut self, provider: String, model: String) {
        self.active_provider = provider;
        self.active_model = model;
    }

    pub fn add_message(&mut self, m_type: MessageType, content: String) {
        use chrono::Local;
        let timestamp = Local::now().format("%H:%M:%S").to_string();
        self.messages.push(Message {
            msg_type: m_type,
            content,
            timestamp,
        });
        self.scroll_chat_to_bottom();
    }

    pub fn scroll_chat_to_bottom(&mut self) {
        if !self.messages.is_empty() {
            self.chat_state.select(Some(self.messages.len() - 1));
        }
    }

    pub fn scroll_up(&mut self) {
        let i = match self.chat_state.selected() {
            Some(i) => {
                if i == 0 {
                    0
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.chat_state.select(Some(i));
    }

    pub fn scroll_down(&mut self) {
        // B2: Guard against usize underflow when message list is empty
        if self.messages.is_empty() { return; }
        let i = match self.chat_state.selected() {
            Some(i) => {
                if i >= self.messages.len() - 1 {
                    self.messages.len() - 1
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.chat_state.select(Some(i));
    }

    pub fn scroll_log_to_bottom(&mut self) {
        if !self.logs.is_empty() {
            self.log_state.select(Some(self.logs.len() - 1));
        }
    }

    pub fn add_log(&mut self, message: String) {
        if self.logs.len() >= 100 {
            self.logs.pop_front();
        }
        self.logs.push_back(message);
        self.scroll_log_to_bottom();
    }

    pub fn set_status(&mut self, mode: Mode) {
        self.status_mode = match mode {
            Mode::Idle => "Idle",
            Mode::Processing => "Thinking...",
            Mode::Completed => "Done",
            Mode::Error => "Error",
        }.to_string();
    }

    pub fn toggle_work_mode(&mut self) {
        self.work_mode = match self.work_mode {
            WorkMode::Plan => WorkMode::Execute,
            WorkMode::Execute => WorkMode::Plan,
        };
        self.add_log(format!("Switched to {:?} mode", self.work_mode));
    }

    pub fn quit(&mut self) {
        self.should_quit = true;
    }

    pub fn clear_messages(&mut self) {
        self.messages.clear();
        self.chat_state.select(None);
        self.add_log("Chat history cleared".to_string());
    }

    pub fn update_ghost_text(&mut self) {
        if self.input.is_empty() {
            self.ghost_text = None;
            return;
        }

        let input_lower = self.input.to_lowercase();

        if input_lower.starts_with('/') {
            let cmd_part = &input_lower[1..];
            // B4: Only ghost-complete when the canonical spec.name starts with what was typed.
            // Aliases are valid shortcuts but should NOT drive ghost-text — that caused the suffix
            // mismatch bug (e.g. typing /acp would ghost-complete mid-word into "commit").
            if let Some(spec) = crate::commands::SLASH_COMMAND_SPECS
                .iter()
                .find(|s| s.name.starts_with(cmd_part))
            {
                if spec.name.len() > cmd_part.len() {
                    self.ghost_text = Some(spec.name[cmd_part.len()..].to_string());
                } else {
                    // Exact match on name — nothing left to complete
                    self.ghost_text = None;
                }
            } else {
                self.ghost_text = None;
            }
        } else {
            self.ghost_text = self.available_skills
                .iter()
                .find(|s| s.to_lowercase().starts_with(&input_lower))
                .map(|s| s[self.input.len()..].to_string());
        }
    }
}

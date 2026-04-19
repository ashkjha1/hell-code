
#[derive(Debug, Clone, PartialEq)]
pub enum SlashCommand {
    Doctor,
    Init,
    Commit { message: Option<String> },
    Status,
    Diff,
    Help { command: Option<String> },
    Clear,
    Model { name: Option<String> },
    Skills,
    Agents,
    Config,
    Version,
    Undo,  // F6: Restore last write from .hell-code/undo_stack/
}

pub struct CommandSpec {
    pub name: &'static str,
    pub aliases: &'static [&'static str],
    pub summary: &'static str,
    pub usage: &'static str,
    pub category: &'static str,
}

pub const SLASH_COMMAND_SPECS: &[CommandSpec] = &[
    CommandSpec {
        name: "doctor",
        aliases: &["d", "check"],
        summary: "Check system health, keys, and git status.",
        usage: "/doctor",
        category: "System",
    },
    CommandSpec {
        name: "init",
        aliases: &["setup"],
        summary: "Initialize workspace with guidance and local config.",
        usage: "/init",
        category: "System",
    },
    CommandSpec {
        name: "commit",
        aliases: &["acp", "c"],
        summary: "AI-powered Git Add-Commit-Push workflow.",
        usage: "/commit [message]",
        category: "Git",
    },
    CommandSpec {
        name: "status",
        aliases: &["st", "s"],
        summary: "Show current workspace and git status.",
        usage: "/status",
        category: "Git",
    },
    CommandSpec {
        name: "diff",
        aliases: &["df"],
        summary: "Show staged or unstaged changes.",
        usage: "/diff",
        category: "Git",
    },
    CommandSpec {
        name: "help",
        aliases: &["h", "?"],
        summary: "Show available commands and usage info.",
        usage: "/help [command]",
        category: "UI",
    },
    CommandSpec {
        name: "clear",
        aliases: &["cls", "clean"],
        summary: "Clear the chat history.",
        usage: "/clear",
        category: "UI",
    },
    CommandSpec {
        name: "model",
        aliases: &["m"],
        summary: "Switch or show the active LLM model.",
        usage: "/model [name]",
        category: "Agent",
    },
    CommandSpec {
        name: "skills",
        aliases: &["sk"],
        summary: "List available skills/tools.",
        usage: "/skills",
        category: "Agent",
    },
    CommandSpec {
        name: "agents",
        aliases: &["ag"],
        summary: "List available agent personas.",
        usage: "/agents",
        category: "Agent",
    },
    CommandSpec {
        name: "config",
        aliases: &["cfg"],
        summary: "Open or show the current configuration.",
        usage: "/config",
        category: "System",
    },
    CommandSpec {
        name: "version",
        aliases: &["v"],
        summary: "Show version information.",
        usage: "/version",
        category: "System",
    },
    CommandSpec {
        name: "undo",
        aliases: &["u"],
        summary: "Restore the last file written by the agent from the undo stack.",
        usage: "/undo",
        category: "System",
    },
];

pub fn parse_slash_command(input: &str) -> Option<SlashCommand> {
    if !input.starts_with('/') {
        return None;
    }

    let input = input.trim_start_matches('/');
    let mut parts = input.splitn(2, ' ');
    let cmd_name = parts.next()?.to_lowercase();
    let arg = parts.next().map(|s| s.trim().to_string());

    // Find the matching spec
    let spec = SLASH_COMMAND_SPECS.iter().find(|s| {
        s.name == cmd_name || s.aliases.contains(&cmd_name.as_str())
    })?;

    match spec.name {
        "doctor" => Some(SlashCommand::Doctor),
        "init" => Some(SlashCommand::Init),
        "commit" => Some(SlashCommand::Commit { message: arg }),
        "status" => Some(SlashCommand::Status),
        "diff" => Some(SlashCommand::Diff),
        "help" => Some(SlashCommand::Help { command: arg }),
        "clear" => Some(SlashCommand::Clear),
        "model" => Some(SlashCommand::Model { name: arg }),
        "skills" => Some(SlashCommand::Skills),
        "agents" => Some(SlashCommand::Agents),
        "config" => Some(SlashCommand::Config),
        "version" => Some(SlashCommand::Version),
        "undo" => Some(SlashCommand::Undo),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_command() {
        assert_eq!(parse_slash_command("/doctor"), Some(SlashCommand::Doctor));
        assert_eq!(parse_slash_command("/d"), Some(SlashCommand::Doctor));
    }

    #[test]
    fn test_parse_command_with_args() {
        assert_eq!(
            parse_slash_command("/commit finished feature"),
            Some(SlashCommand::Commit { message: Some("finished feature".to_string()) })
        );
        assert_eq!(
            parse_slash_command("/help model"),
            Some(SlashCommand::Help { command: Some("model".to_string()) })
        );
    }

    #[test]
    fn test_parse_invalid_command() {
        assert_eq!(parse_slash_command("/nonexistent"), None);
        assert_eq!(parse_slash_command("not a command"), None);
    }
}

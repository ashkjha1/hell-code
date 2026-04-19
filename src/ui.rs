use ratatui::{
    prelude::*,
    widgets::*,
};
use crate::app::{App, InputMode, WorkMode};
use crate::theme::{Theme, get_message_style};

pub fn render(f: &mut Frame, app: &App) {
    let theme = Theme::default();
    
    // Main Container
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),   // Header/Logo
            Constraint::Min(10),     // Center Area (Chat [+ Logs])
            Constraint::Length(3),   // Input
            Constraint::Length(1),   // Status
        ])
        .split(f.size());

    // --- 1. LOGO ---
    render_logo(f, chunks[0], &theme);

    // --- 2. CENTER AREA (Handling Verbose Split) ---
    if app.verbose {
        let center_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(50), // Chat
                Constraint::Percentage(50), // Logs
            ])
            .split(chunks[1]);
        
        render_chat_area(f, center_chunks[0], app, &theme);
        render_log_area(f, center_chunks[1], app, &theme);
    } else {
        render_chat_area(f, chunks[1], app, &theme);
    }

    // --- 3. INPUT AREA ---
    render_input_area(f, chunks[2], app, &theme);

    // --- 4. STATUS BAR ---
    render_status_bar(f, chunks[3], app, &theme);

    // --- 5. HELP MODAL (OVERLAY) ---
    if app.show_help {
        render_help_modal(f, &theme);
    }
}

fn render_logo(f: &mut Frame, area: Rect, theme: &Theme) {
    let logo_text = vec![
        Line::from(vec![
            Span::styled(" ▟████▙ ", Style::default().fg(theme.mauve)),
            Span::styled(" HELL ", Style::default().fg(theme.text).add_modifier(Modifier::BOLD)),
            Span::styled("CODE", Style::default().fg(theme.mauve).add_modifier(Modifier::BOLD)),
            Span::styled(" ▟████▙ ", Style::default().fg(theme.mauve)),
        ]),
    ];
    
    f.render_widget(
        Paragraph::new(logo_text)
            .alignment(Alignment::Center)
            .block(Block::default().padding(Padding::vertical(1))),
        area
    );
}

fn render_chat_area(f: &mut Frame, area: Rect, app: &App, theme: &Theme) {
    let mut messages: Vec<ListItem> = Vec::new();
    
    for m in &app.messages {
        let (name, color, icon) = get_message_style(&m.msg_type, theme);
        let mut lines = Vec::new();
        
        // Header line
        lines.push(Line::from(vec![
            Span::styled(format!(" {} ", icon), Style::default().fg(color)),
            Span::styled(name.to_string(), Style::default().fg(color).add_modifier(Modifier::BOLD)),
            Span::styled(format!(" ({})", m.timestamp), Style::default().fg(theme.gray).add_modifier(Modifier::ITALIC)),
        ]));

        let mut in_code_block = false;

        // Content
        for line in m.content.lines() {
            let trimmed = line.trim();
            if trimmed.starts_with("```") {
                in_code_block = !in_code_block;
                continue;
            }

            if in_code_block {
                lines.push(Line::from(vec![
                    Span::raw("   "),
                    Span::styled(line, Style::default().fg(Color::Yellow).bg(theme.charcoal)),
                ]));
            } else {
                let parsed_spans = parse_markdown_line(line, theme);
                let mut line_spans = vec![Span::raw("   ")];
                line_spans.extend(parsed_spans);
                lines.push(Line::from(line_spans));
            }
        }
        
        lines.push(Line::from("")); // Spacer
        messages.push(ListItem::new(lines));
    }

    let chat_list = List::new(messages)
        .block(Block::default()
            .borders(Borders::LEFT | Borders::RIGHT)
            .border_style(Style::default().fg(theme.surface1))
            .bg(theme.base)
            .title(Span::styled(" CONVERSATION ", Style::default().fg(theme.mauve).add_modifier(Modifier::BOLD))))
        .style(Style::default().fg(theme.text))
        .highlight_style(Style::default().add_modifier(Modifier::REVERSED))
        .highlight_symbol(">> ");

    f.render_stateful_widget(chat_list, area, &mut app.chat_state.clone());
}

fn parse_markdown_line<'a>(line: &'a str, theme: &Theme) -> Vec<Span<'a>> {
    let mut spans = Vec::new();
    
    // This is a very basic "lite" parser for **bold**, *italic*, and `code`
    // It doesn't handle nested overlapping tags perfectly but works for well-formed markdown
    let mut remaining = line;
    
    while !remaining.is_empty() {
        let bold_idx = remaining.find("**");
        let italic_idx = remaining.find('*');
        let code_idx = remaining.find('`');
        
        let mut indices = vec![];
        if let Some(i) = bold_idx { indices.push((i, "bold")); }
        if let Some(i) = italic_idx { indices.push((i, "italic")); }
        if let Some(i) = code_idx { indices.push((i, "code")); }
        
        indices.sort_by_key(|(i, _)| *i);
        
        if let Some((idx, tag)) = indices.first() {
            // Push text before the tag
            if *idx > 0 {
                spans.push(Span::styled(&remaining[..*idx], Style::default().fg(theme.text_bright)));
            }
            
            let after_tag = &remaining[*idx..];
            match *tag {
                "bold" => {
                    if let Some(end_idx) = after_tag[2..].find("**") {
                        let content = &after_tag[2..end_idx + 2];
                        spans.push(Span::styled(content, Style::default().fg(theme.text_bright).add_modifier(Modifier::BOLD)));
                        remaining = &after_tag[end_idx + 4..];
                    } else {
                        spans.push(Span::raw("**"));
                        remaining = &after_tag[2..];
                    }
                }
                "italic" => {
                    if let Some(end_idx) = after_tag[1..].find('*') {
                        let content = &after_tag[1..end_idx + 1];
                        spans.push(Span::styled(content, Style::default().fg(theme.text_bright).add_modifier(Modifier::ITALIC)));
                        remaining = &after_tag[end_idx + 2..];
                    } else {
                        spans.push(Span::raw("*"));
                        remaining = &after_tag[1..];
                    }
                }
                "code" => {
                    if let Some(end_idx) = after_tag[1..].find('`') {
                        let content = &after_tag[1..end_idx + 1];
                        spans.push(Span::styled(content, Style::default().fg(Color::Yellow)));
                        remaining = &after_tag[end_idx + 2..];
                    } else {
                        spans.push(Span::raw("`"));
                        remaining = &after_tag[1..];
                    }
                }
                _ => {}
            }
        } else {
            // No more tags
            spans.push(Span::styled(remaining, Style::default().fg(theme.text_bright)));
            break;
        }
    }
    
    if spans.is_empty() && !line.is_empty() {
        spans.push(Span::styled(line, Style::default().fg(theme.text_bright)));
    }
    
    spans
}

fn render_log_area(f: &mut Frame, area: Rect, app: &App, theme: &Theme) {
    let logs: Vec<ListItem> = app.logs.iter().map(|log| {
        ListItem::new(Line::from(vec![
            Span::styled(" > ", Style::default().fg(theme.gray)),
            Span::styled(log, Style::default().fg(theme.gray)),
        ]))
    }).collect();

    let log_list = List::new(logs)
        .block(Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(theme.surface1))
            .bg(theme.mantle)
            .title(Span::styled(" RAW DEBUG LOGS ", Style::default().fg(theme.yellow).add_modifier(Modifier::BOLD))))
        .style(Style::default().fg(theme.overlay1));

    f.render_stateful_widget(log_list, area, &mut app.log_state.clone());
}

fn render_input_area(f: &mut Frame, area: Rect, app: &App, theme: &Theme) {
    let is_editing = app.input_mode == InputMode::Editing;
    
    let border_color = if is_editing { theme.mauve } else { theme.surface1 };
    
    let input_block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(border_color))
        .bg(theme.base)
        .title(Span::styled(
            if is_editing { " [ EDITING ] " } else { " [ NORMAL ] " },
            Style::default().fg(border_color).add_modifier(Modifier::BOLD)
        ));

    let mut input_spans = vec![
        Span::styled(" λ ", Style::default().fg(theme.lavender).add_modifier(Modifier::BOLD)),
        Span::raw(&app.input),
    ];

    if let Some(ref ghost) = app.ghost_text {
        input_spans.push(Span::styled(ghost, Style::default().fg(theme.gray).add_modifier(Modifier::ITALIC)));
    }

    let input_para = Paragraph::new(Line::from(input_spans))
        .block(input_block);

    f.render_widget(input_para, area);

    if is_editing {
        // B3: Use chars().count() for unicode safety — .len() returns bytes, not characters
        f.set_cursor(
            area.x + app.input.chars().count() as u16 + 4,
            area.y + 1,
        );
    }
}

fn render_status_bar(f: &mut Frame, area: Rect, app: &App, theme: &Theme) {
    let work_mode_color = match app.work_mode {
        WorkMode::Plan => theme.sky,
        WorkMode::Execute => theme.red,
    };

    let spinner = ["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];
    let frame = (app.tick_count % spinner.len() as u64) as usize;
    let is_processing = app.status_mode == "Thinking...";

    let status_content = Line::from(vec![
        Span::styled(format!(" {:?} ", app.work_mode), Style::default().fg(theme.base).bg(work_mode_color).add_modifier(Modifier::BOLD)),
        Span::styled("", Style::default().fg(work_mode_color).bg(theme.surface1)),
        Span::styled(format!(" {} ", app.active_model), Style::default().fg(theme.text).bg(theme.surface1)),
        Span::styled("", Style::default().fg(theme.surface1).bg(theme.mantle)),
        Span::styled(format!(" {} ", if is_processing { spinner[frame] } else { "●" }), Style::default().fg(if is_processing { theme.mauve } else { theme.overlay1 }).add_modifier(Modifier::BOLD)),
        Span::styled(format!(" {} ", app.status_mode), Style::default().fg(theme.subtext0)),
        Span::styled(format!(" | [v]:Verbose [?]:Help "), Style::default().fg(theme.overlay1)),
        Span::styled(format!(" | Task: {} ", app.current_task.as_deref().unwrap_or("None")), Style::default().fg(theme.overlay1)),
    ]);

    f.render_widget(Paragraph::new(status_content).bg(theme.mantle), area);
}

fn render_help_modal(f: &mut Frame, theme: &Theme) {
    // B1: Larger modal to fit all slash commands
    let area = centered_rect(65, 80, f.size());
    f.render_widget(Clear, area);

    let mut help_text = vec![
        Line::from(vec![Span::styled("⌨  GLOBAL KEYBINDINGS", Style::default().fg(theme.mauve).add_modifier(Modifier::BOLD))]),
        Line::from(""),
        Line::from(vec![Span::styled(" [i]     ", Style::default().fg(theme.yellow)), Span::raw("Enter Editing Mode")]),
        Line::from(vec![Span::styled(" [ESC]   ", Style::default().fg(theme.yellow)), Span::raw("Back to Normal Mode")]),
        Line::from(vec![Span::styled(" [TAB]   ", Style::default().fg(theme.yellow)), Span::raw("Toggle Plan / Execute Mode")]),
        Line::from(vec![Span::styled(" [↑ ↓]   ", Style::default().fg(theme.yellow)), Span::raw("Scroll conversation history")]),
        Line::from(vec![Span::styled(" [→]     ", Style::default().fg(theme.yellow)), Span::raw("Accept ghost-text suggestion")]),
        Line::from(vec![Span::styled(" [v]     ", Style::default().fg(theme.yellow)), Span::raw("Toggle verbose log panel")]),
        Line::from(vec![Span::styled(" [?]     ", Style::default().fg(theme.yellow)), Span::raw("Toggle this help modal")]),
        Line::from(vec![Span::styled(" [q]     ", Style::default().fg(theme.yellow)), Span::raw("Quit")]),
        Line::from(""),
        Line::from(vec![Span::styled("🚀  SLASH COMMANDS", Style::default().fg(theme.mauve).add_modifier(Modifier::BOLD))]),
        Line::from(""),
    ];

    // B1: Dynamically render commands from registry, grouped by category
    let categories = ["System", "Git", "Agent", "UI"];
    for category in categories {
        let cmds: Vec<_> = crate::commands::SLASH_COMMAND_SPECS
            .iter()
            .filter(|s| s.category == category)
            .collect();
        if cmds.is_empty() { continue; }

        help_text.push(Line::from(vec![
            Span::styled(
                format!("  — {} ", category),
                Style::default().fg(theme.overlay1).add_modifier(Modifier::ITALIC),
            ),
        ]));
        for spec in cmds {
            help_text.push(Line::from(vec![
                Span::styled(
                    format!(" /{:<10}", spec.name),
                    Style::default().fg(theme.teal).add_modifier(Modifier::BOLD),
                ),
                Span::styled(spec.summary, Style::default().fg(theme.subtext0)),
            ]));
        }
        help_text.push(Line::from(""));
    }

    help_text.push(Line::from(vec![
        Span::styled(" Press [?] or [ESC] to close", Style::default().fg(theme.gray).add_modifier(Modifier::ITALIC)),
    ]));

    let help_block = Block::default()
        .title(" HELL-CODE COMMAND CENTER ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme.mauve))
        .bg(theme.crust)
        .padding(Padding::uniform(1));

    f.render_widget(
        Paragraph::new(help_text).block(help_block).style(Style::default().fg(theme.text)),
        area,
    );
}

fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}
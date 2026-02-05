use ratatui::{
    layout::Alignment,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph, Wrap},
    Frame,
};

use crate::app::{App, InputMode};

/// Main entry point for drawing the TUI. Dispatches to individual pane drawers.
pub fn draw(f: &mut Frame, app: &mut App) {
    let vertical_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Length(3), // Header
                Constraint::Min(10),   // Main Content (List + Preview)
                Constraint::Length(3), // Search
                Constraint::Length(5), // Status/Selected/Shortcuts
            ]
            .as_ref(),
        )
        .split(f.area());

    // Header
    let header = Paragraph::new("Welcome to autogitignore")
        .style(Style::default().fg(Color::Magenta).add_modifier(Modifier::BOLD))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Magenta)),
        )
        .alignment(Alignment::Center);
    f.render_widget(header, vertical_chunks[0]);

    // Main Content: Split Horizontal (List | Preview)
    let main_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
        .split(vertical_chunks[1]);

    draw_list_pane(f, app, main_chunks[0]);
    draw_preview_pane(f, app, main_chunks[1]);

    // Search input
    draw_search_pane(f, app, vertical_chunks[2]);

    // Status / Selected
    draw_status_pane(f, app, vertical_chunks[3]);

    if let InputMode::Confirm = app.input_mode {
        draw_confirm_modal(f, app);
    }
}

/// Renders the left pane containing the list of filtered templates.
fn draw_list_pane(f: &mut Frame, app: &mut App, area: Rect) {
    let items: Vec<ListItem> = if app.is_loading && app.filtered_templates.is_empty() {
        vec![ListItem::new("Fetching templates from gitignore.io...")
            .style(Style::default().fg(Color::DarkGray).add_modifier(Modifier::ITALIC))]
    } else if app.filtered_templates.is_empty() {
        vec![ListItem::new("No templates found.").style(Style::default().fg(Color::Yellow))]
    } else {
        app.filtered_templates
            .iter()
            .map(|t| {
                let is_selected = app.selected_templates.contains(t);
                let content = if is_selected {
                    format!("[X] {}", t)
                } else {
                    format!("[ ] {}", t)
                };

                let style = if is_selected {
                    Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)
                } else {
                    Style::default()
                };
                ListItem::new(content).style(style)
            })
            .collect()
    };

    let mut state = ListState::default();
    if app.filtered_templates.is_empty() {
        state.select(None);
    } else {
        state.select(Some(app.highlighted_index));
    }

    let list = List::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Matching Templates ")
                .border_style(Style::default().fg(Color::Cyan)),
        )
        .highlight_style(
            Style::default()
                .bg(Color::Blue)
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol("▶ ");

    f.render_stateful_widget(list, area, &mut state);
}

/// Renders the right pane showing the preview of highlighted or combined templates.
fn draw_preview_pane(f: &mut Frame, app: &mut App, area: Rect) {
    let mode_str = match app.preview_mode {
        crate::app::PreviewMode::Highlighted => " [HIGHLIGHT] ",
        crate::app::PreviewMode::Combined => " [COMBINED] ",
    };

    let title = format!(" Preview {} ", mode_str);
    let content = app.get_combined_preview();
    let content_height = area.height.saturating_sub(2);
    app.set_preview_height(content_height);
    let preview = Paragraph::new(content)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(Span::styled(
                    title,
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                ))
                .border_style(Style::default().fg(Color::Yellow)),
        )
        .wrap(Wrap { trim: false })
        .scroll((app.preview_scroll, 0));

    f.render_widget(preview, area);
}

/// Renders the search input field.
fn draw_search_pane(f: &mut Frame, app: &mut App, area: Rect) {
    let input_style = if let InputMode::Editing = app.input_mode {
        Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Color::DarkGray)
    };

    let title = if let InputMode::Editing = app.input_mode {
        Span::styled(" Search (Typing...) ", Style::default().fg(Color::Cyan))
    } else {
        Span::styled(
            " Search (Press '/' or 'i' to browse) ",
            Style::default().fg(Color::DarkGray),
        )
    };

    let input = Paragraph::new(app.search_query.as_str())
        .style(input_style)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(title)
                .border_style(input_style),
        );
    f.render_widget(input, area);

    if let InputMode::Editing = app.input_mode {
        let cursor_x = area.x.saturating_add(1).saturating_add(app.search_query.len() as u16);
        let max_x = area.x.saturating_add(area.width.saturating_sub(1));
        let cursor_x = cursor_x.min(max_x);
        f.set_cursor_position((cursor_x, area.y + 1));
    }
}

/// Renders the bottom status bar including selected templates summary and key shortcuts.
fn draw_status_pane(f: &mut Frame, app: &mut App, area: Rect) {
    let selected_count = app.selected_templates.len();
    let selected_names = app.get_selected_names_summary();

    let mut status_lines = Vec::new();

    // Line 1: Success/Error or Selection Info
    if let Some(msg) = &app.notification {
        status_lines.push(Line::from(vec![
            Span::styled(
                " SUCCESS ",
                Style::default()
                    .bg(Color::Green)
                    .fg(Color::Black)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(" "),
            Span::styled(msg, Style::default().fg(Color::LightGreen)),
        ]));
    } else if let Some(err) = &app.error {
        status_lines.push(Line::from(vec![
            Span::styled(
                " ERROR ",
                Style::default()
                    .bg(Color::Red)
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(" "),
            Span::styled(err, Style::default().fg(Color::LightRed)),
        ]));
    } else {
        let mut spans = vec![
            Span::styled(
                format!(" SELECTED ({}): ", selected_count),
                Style::default()
                    .bg(Color::Cyan)
                    .fg(Color::Black)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(" "),
        ];

        if selected_count > 0 {
            spans.push(Span::styled(selected_names, Style::default().fg(Color::Green)));
        } else {
            spans.push(Span::styled("None", Style::default().fg(Color::DarkGray)));
        }
        status_lines.push(Line::from(spans));
    }

    status_lines.push(Line::from("")); // Spacer

    // Line 3: Shortcuts (Beautifully formatted)
    let shortcuts = vec![
        ("SPACE", "Select"),
        ("/", "Search"),
        ("P", "Toggle Mode"),
        ("ALT+J/K", "Scroll Preview"),
        ("CTRL+S", "Save"),
        ("ENTER", "Save&Quit"),
        ("Q", "Quit"),
    ];

    let mut shortcut_spans = Vec::new();
    for (i, (key, desc)) in shortcuts.iter().enumerate() {
        if i > 0 {
            shortcut_spans.push(Span::raw("  "));
        }
        shortcut_spans.push(Span::styled(
            format!(" {} ", key),
            Style::default()
                .bg(Color::DarkGray)
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        ));
        shortcut_spans.push(Span::raw(format!(" {}", desc)));
    }
    status_lines.push(Line::from(shortcut_spans));

    let status = Paragraph::new(status_lines)
        .block(Block::default().borders(Borders::ALL).title(" Info & Controls "));
    f.render_widget(status, area);
}

/// Renders the centered confirmation modal for handling existing .gitignore files.
fn draw_confirm_modal(f: &mut Frame, app: &mut App) {
    let area = f.area();
    let block = Block::default()
        .title(" .gitignore already exists! ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD));

    let modal_area = centered_rect(50, 40, area);
    f.render_widget(ratatui::widgets::Clear, modal_area);

    let text = vec![
        Line::from(""),
        Line::from(vec![
            Span::raw("An existing "),
            Span::styled(
                ".gitignore",
                Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
            ),
            Span::raw(" file was found."),
        ]),
        Line::from(""),
        Line::from("Choose an action:"),
        Line::from(""),
        Line::from(vec![
            Span::styled(
                " [A] Append ",
                if app.confirm_action == Some(crate::app::ConfirmAction::Append) {
                    Style::default()
                        .bg(Color::Green)
                        .fg(Color::Black)
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(Color::Green)
                },
            ),
            Span::raw("    "),
            Span::styled(
                " [O] Overwrite ",
                if app.confirm_action == Some(crate::app::ConfirmAction::Overwrite) {
                    Style::default()
                        .bg(Color::Red)
                        .fg(Color::White)
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(Color::Red)
                },
            ),
        ]),
        Line::from(""),
        Line::from(""),
        Line::from(vec![Span::styled(
            " Use Left/Right Arrow or A/O to select, Enter to confirm ",
            Style::default().fg(Color::DarkGray).add_modifier(Modifier::ITALIC),
        )]),
        Line::from(""),
        Line::from(vec![Span::styled(
            " Press [ESC] to cancel ",
            Style::default().fg(Color::DarkGray),
        )]),
    ];

    let paragraph = Paragraph::new(text)
        .block(block)
        .alignment(Alignment::Center)
        .wrap(Wrap { trim: true });

    f.render_widget(paragraph, modal_area);
}

/// Helper function to create a centered rectangle for popups/modals.
fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Percentage((100 - percent_y) / 2),
                Constraint::Percentage(percent_y),
                Constraint::Percentage((100 - percent_y) / 2),
            ]
            .as_ref(),
        )
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints(
            [
                Constraint::Percentage((100 - percent_x) / 2),
                Constraint::Percentage(percent_x),
                Constraint::Percentage((100 - percent_x) / 2),
            ]
            .as_ref(),
        )
        .split(popup_layout[1])[1]
}

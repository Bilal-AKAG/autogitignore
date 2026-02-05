mod api;
mod app;
mod models;
mod ui;

use crate::models::CacheData;
use crate::ui::draw;
use anyhow::Result;
use app::{App, InputMode};
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};
use std::{io, time::Duration};
use tokio::sync::mpsc;

enum AppEvent {
    Tick,
    Key(event::KeyEvent),
    DataLoaded(CacheData),
    Error(String),
}

struct TerminalSession {
    terminal: Terminal<CrosstermBackend<io::Stdout>>,
}

impl TerminalSession {
    fn new() -> Result<Self> {
        enable_raw_mode()?;
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
        let backend = CrosstermBackend::new(stdout);
        let terminal = Terminal::new(backend)?;
        Ok(Self { terminal })
    }

    fn terminal_mut(&mut self) -> &mut Terminal<CrosstermBackend<io::Stdout>> {
        &mut self.terminal
    }
}

impl Drop for TerminalSession {
    fn drop(&mut self) {
        let _ = disable_raw_mode();
        let _ = execute!(
            self.terminal.backend_mut(),
            LeaveAlternateScreen,
            DisableMouseCapture
        );
        let _ = self.terminal.show_cursor();
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let mut session = TerminalSession::new()?;
    let mut app = App::new();
    let (tx, mut rx) = mpsc::channel(100);

    // Sync / Cache logic
    let client = crate::api::ApiClient::new()?;
    let tx_c = tx.clone();

    // Check cache
    if let Some(cache) = client.load_cache() {
        let _ = tx_c.send(AppEvent::DataLoaded(cache)).await;
    } else {
        // FULL SYNC from Toptal
        tokio::spawn(async move {
            match client.fetch_all_data().await {
                Ok(cache) => {
                    let _ = client.save_cache(&cache);
                    let _ = tx_c.send(AppEvent::DataLoaded(cache)).await;
                }
                Err(e) => {
                    let _ = tx_c.send(AppEvent::Error(e.to_string())).await;
                }
            }
        });
    }

    // Event loop thread
    let tx_c = tx.clone();
    tokio::spawn(async move {
        loop {
            if event::poll(Duration::from_millis(100)).unwrap_or(false) {
                match event::read() {
                    Ok(Event::Key(key)) if key.kind == KeyEventKind::Press => {
                        let _ = tx_c.send(AppEvent::Key(key)).await;
                    }
                    Ok(_) => {}
                    Err(err) => {
                        let _ = tx_c.send(AppEvent::Error(err.to_string())).await;
                    }
                }
            }
            let _ = tx_c.send(AppEvent::Tick).await;
        }
    });

    'main_loop: loop {
        session.terminal_mut().draw(|f| draw(f, &mut app))?;

        if let Some(ev) = rx.recv().await {
            match ev {
                AppEvent::Tick => {}
                AppEvent::Error(e) => {
                    app.error = Some(e);
                    app.is_loading = false;
                }
                AppEvent::DataLoaded(cache) => {
                    app.set_templates(cache.templates);
                    app.template_contents = cache.contents;
                    app.is_loading = false;
                    app.apply_filter();
                }
                AppEvent::Key(key) => match app.input_mode {
                    InputMode::Editing => match key.code {
                        KeyCode::Char(c) => {
                            app.notification = None;
                            app.error = None;
                            app.search_query.push(c);
                            app.apply_filter();
                        }
                        KeyCode::Backspace => {
                            app.notification = None;
                            app.error = None;
                            app.search_query.pop();
                            app.apply_filter();
                        }
                        KeyCode::Esc | KeyCode::Enter => {
                            app.input_mode = InputMode::Normal;
                        }
                        KeyCode::Down => app.next(),
                        KeyCode::Up => app.previous(),
                        _ => {}
                    },
                    InputMode::Normal => match key.code {
                        KeyCode::Char('i') | KeyCode::Char('/') => {
                            app.notification = None;
                            app.error = None;
                            app.input_mode = InputMode::Editing;
                        }
                        KeyCode::Char('q') | KeyCode::Esc => {
                            break;
                        }
                        KeyCode::Down | KeyCode::Char('j')
                            if key.modifiers.contains(KeyModifiers::ALT) =>
                        {
                            let line_count = app.get_preview_line_count();
                            if (app.preview_scroll as usize) < line_count.saturating_sub(2) {
                                app.preview_scroll = app.preview_scroll.saturating_add(1);
                            }
                        }
                        KeyCode::Up | KeyCode::Char('k')
                            if key.modifiers.contains(KeyModifiers::ALT) =>
                        {
                            app.preview_scroll = app.preview_scroll.saturating_sub(1);
                        }
                        KeyCode::Down | KeyCode::Char('j') => app.next(),
                        KeyCode::Up | KeyCode::Char('k') => app.previous(),
                        KeyCode::Char(' ') => app.toggle_selection(),
                        KeyCode::Char('p') => {
                            app.preview_mode = match app.preview_mode {
                                crate::app::PreviewMode::Highlighted => {
                                    crate::app::PreviewMode::Combined
                                }
                                crate::app::PreviewMode::Combined => {
                                    crate::app::PreviewMode::Highlighted
                                }
                            };
                            app.preview_scroll = 0;
                        }
                        KeyCode::PageDown => {
                            let line_count = app.get_preview_line_count();
                            let target = app.preview_scroll.saturating_add(10);
                            if (target as usize) < line_count {
                                app.preview_scroll = target;
                            } else {
                                app.preview_scroll = line_count.saturating_sub(2) as u16;
                            }
                        }
                        KeyCode::PageUp => {
                            app.preview_scroll = app.preview_scroll.saturating_sub(10);
                        }
                        KeyCode::Enter | KeyCode::Char('s')
                            if key.modifiers.contains(KeyModifiers::CONTROL) =>
                        {
                            if app.selected_templates.is_empty() {
                                app.error = Some("No templates selected!".to_string());
                            } else {
                                app.error = Some("Save not implemented yet.".to_string());
                            }
                        }
                        _ => {}
                    },
                    InputMode::Confirm => match key.code {
                        KeyCode::Char('a') | KeyCode::Left => {
                            app.confirm_action = Some(crate::app::ConfirmAction::Append);
                        }
                        KeyCode::Char('o') | KeyCode::Right => {
                            app.confirm_action = Some(crate::app::ConfirmAction::Overwrite);
                        }
                        KeyCode::Enter => {
                            app.error = Some("Save not implemented yet.".to_string());
                            app.input_mode = InputMode::Normal;
                        }
                        KeyCode::Esc => {
                            app.error = None;
                            app.notification = None;
                            app.input_mode = InputMode::Normal;
                        }
                        _ => {}
                    },
                },
            }
        }
    }

    Ok(())
}

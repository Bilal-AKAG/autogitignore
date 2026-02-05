mod api;
mod app;
mod gitignore;
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
use std::{io, path::PathBuf, time::Duration};
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
    let output_dir = parse_output_dir()?;
    let mut app = App::new(output_dir);
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
                            let max_scroll = app.max_preview_scroll();
                            if app.preview_scroll < max_scroll {
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
                            let max_scroll = app.max_preview_scroll();
                            let target = app.preview_scroll.saturating_add(10);
                            app.preview_scroll = target.min(max_scroll);
                        }
                        KeyCode::PageUp => {
                            app.preview_scroll = app.preview_scroll.saturating_sub(10);
                        }
                        KeyCode::Enter => {
                            // Save and Quit
                            if !app.selected_templates.is_empty() {
                                app.notification = None;
                                app.error = None;
                                app.should_quit_after_save = true;
                                if app.gitignore_exists() {
                                    app.input_mode = InputMode::Confirm;
                                    app.confirm_action = Some(crate::app::ConfirmAction::Append);
                                } else {
                                    let content = app.generate_gitignore_content();
                                    if gitignore::write_gitignore(&app.gitignore_path(), &content, gitignore::WriteMode::Overwrite).is_ok() {
                                        break 'main_loop;
                                    }
                                }
                            } else {
                                app.error = Some("No templates selected!".to_string());
                            }
                        }
                        KeyCode::Char('s') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                            // Save
                            if !app.selected_templates.is_empty() {
                                app.notification = None;
                                app.error = None;
                                app.should_quit_after_save = false;
                                if app.gitignore_exists() {
                                    app.input_mode = InputMode::Confirm;
                                    app.confirm_action = Some(crate::app::ConfirmAction::Append);
                                } else {
                                    let content = app.generate_gitignore_content();
                                    match gitignore::write_gitignore(&app.gitignore_path(), &content, gitignore::WriteMode::Overwrite) {
                                        Ok(_) => app.notification = Some("Successfully created .gitignore!".to_string()),
                                        Err(e) => app.error = Some(format!("Failed to write: {}", e)),
                                    }
                                }
                            } else {
                                app.error = Some("No templates selected!".to_string());
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
                            let mode = match app.confirm_action {
                                Some(crate::app::ConfirmAction::Append) => gitignore::WriteMode::Append,
                                _ => gitignore::WriteMode::Overwrite,
                            };
                            let content = app.generate_gitignore_content();
                            let should_quit = app.should_quit_after_save;
                            match gitignore::write_gitignore(&app.gitignore_path(), &content, mode) {
                                Ok(_) => {
                                    if should_quit {
                                        break 'main_loop;
                                    }
                                    app.notification = Some(format!(
                                        "Successfully {}ed .gitignore!",
                                        if let gitignore::WriteMode::Append = mode {
                                            "append"
                                        } else {
                                            "overwrit"
                                        }
                                    ));
                                    app.input_mode = InputMode::Normal;
                                }
                                Err(e) => {
                                    app.error = Some(format!("Failed to write: {}", e));
                                    app.input_mode = InputMode::Normal;
                                }
                            }
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

fn parse_output_dir() -> Result<PathBuf> {
    let mut args = std::env::args().skip(1);
    let mut output_dir: Option<PathBuf> = None;

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "-d" | "--dir" => {
                let value = args
                    .next()
                    .ok_or_else(|| anyhow::anyhow!("--dir requires a path"))?;
                output_dir = Some(PathBuf::from(value));
            }
            _ => {
                if output_dir.is_some() {
                    return Err(anyhow::anyhow!("Unexpected argument: {}", arg));
                }
                output_dir = Some(PathBuf::from(arg));
            }
        }
    }

    let cwd = std::env::current_dir()?;
    let dir = output_dir.map_or(cwd.clone(), |path| {
        if path.is_absolute() {
            path
        } else {
            cwd.join(path)
        }
    });

    if !dir.is_dir() {
        return Err(anyhow::anyhow!("Target path is not a directory: {}", dir.display()));
    }

    Ok(dir)
}

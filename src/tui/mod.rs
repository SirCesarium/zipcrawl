pub mod app;
pub mod ui;

use crate::archive::ZipManager;
use crate::errors::ZipCrawlError;
use crate::tui::app::{App, InputMode};
use crossterm::event::{
    self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyModifiers, MouseEventKind,
};
use crossterm::terminal::{
    EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode,
};
use ratatui::{Terminal, backend::CrosstermBackend};
use std::io;

#[allow(clippy::too_many_lines)]
pub fn handle(manager: &mut ZipManager) -> Result<(), ZipCrawlError> {
    enable_raw_mode().ok();
    let mut stdout = io::stdout();
    crossterm::execute!(stdout, EnterAlternateScreen, EnableMouseCapture).ok();

    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend).map_err(|e| ZipCrawlError::IoError {
        path: "tui".into(),
        source: e,
    })?;

    let mut app = App::new(manager);

    loop {
        terminal.draw(|f| ui::draw(f, &app)).ok();

        match event::read() {
            Ok(Event::Key(key)) => {
                if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('c') {
                    break;
                }

                match app.input_mode {
                    InputMode::Normal => match key.code {
                        KeyCode::Char('/') => app.input_mode = InputMode::Search,
                        KeyCode::Char('j') | KeyCode::Down => {
                            if !app.current_entries.is_empty() {
                                app.selected_index = (app.selected_index + 1)
                                    .min(app.current_entries.len().saturating_sub(1));
                                if app.show_preview {
                                    app.load_preview();
                                }
                            }
                        }
                        KeyCode::Char('k') | KeyCode::Up => {
                            app.selected_index = app.selected_index.saturating_sub(1);
                            if app.show_preview {
                                app.load_preview();
                            }
                        }
                        KeyCode::Char('l') | KeyCode::Right | KeyCode::Enter => {
                            app.enter_directory();
                        }
                        KeyCode::Char('h') | KeyCode::Left | KeyCode::Backspace => app.go_back(),
                        KeyCode::Char('o') => {
                            app.show_preview = !app.show_preview;
                            if app.show_preview {
                                app.load_preview();
                            }
                        }
                        _ => {}
                    },
                    InputMode::Search => match key.code {
                        KeyCode::Enter | KeyCode::Esc => app.input_mode = InputMode::Normal,
                        KeyCode::Char(c) => {
                            app.search_query.push(c);
                            app.apply_filter();
                        }
                        KeyCode::Backspace => {
                            app.search_query.pop();
                            app.apply_filter();
                        }
                        _ => {}
                    },
                }
            }
            Ok(Event::Mouse(mouse)) => {
                let is_preview_area = if app.show_preview {
                    let width = terminal.size().map(|s| s.width).unwrap_or(0);
                    mouse.column > (width * 40 / 100)
                } else {
                    false
                };

                match mouse.kind {
                    MouseEventKind::ScrollDown => {
                        if is_preview_area {
                            app.preview_scroll = app.preview_scroll.saturating_add(2);
                        } else {
                            app.selected_index = (app.selected_index + 1)
                                .min(app.current_entries.len().saturating_sub(1));
                            if app.show_preview {
                                app.load_preview();
                            }
                        }
                    }
                    MouseEventKind::ScrollUp => {
                        if is_preview_area {
                            app.preview_scroll = app.preview_scroll.saturating_sub(2);
                        } else {
                            app.selected_index = app.selected_index.saturating_sub(1);
                            if app.show_preview {
                                app.load_preview();
                            }
                        }
                    }
                    _ => {}
                }
            }
            _ => {}
        }
    }

    disable_raw_mode().ok();
    crossterm::execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )
    .ok();
    Ok(())
}

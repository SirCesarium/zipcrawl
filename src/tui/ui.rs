use crate::tui::app::{App, InputMode};
use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, List, ListItem, Paragraph, Wrap};
use std::convert::TryInto;

pub fn draw(f: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(0),
            Constraint::Length(1),
        ])
        .split(f.area());

    let title = if app.current_path.is_empty() {
        " root ".to_string()
    } else {
        format!(" {} ", app.current_path)
    };

    let search_text = format!("  {}", app.search_query);
    let search_bar = Paragraph::new(search_text.as_str())
        .block(Block::default().borders(Borders::ALL).title(title))
        .style(match app.input_mode {
            InputMode::Search => Style::default().fg(Color::Cyan),
            InputMode::Normal => Style::default(),
        });
    f.render_widget(search_bar, chunks[0]);

    if matches!(app.input_mode, InputMode::Search) {
        let query_len: u16 = app.search_query.len().try_into().unwrap_or(u16::MAX);
        let cursor_x = chunks[0].x.saturating_add(query_len).saturating_add(4);

        f.set_cursor_position(Position::new(
            cursor_x.min(chunks[0].right().saturating_sub(1)),
            chunks[0].y + 1,
        ));
    }

    let body_constraints = if app.show_preview {
        [Constraint::Percentage(40), Constraint::Percentage(60)]
    } else {
        [Constraint::Percentage(100), Constraint::Percentage(0)]
    };
    let body = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(body_constraints)
        .split(chunks[1]);

    let items: Vec<ListItem> = app
        .current_entries
        .iter()
        .enumerate()
        .map(|(i, e)| {
            let (icon, color) = if e.is_dir {
                (" ", Color::Yellow)
            } else {
                (" ", Color::White)
            };
            let name = e.name.strip_prefix(&app.current_path).unwrap_or(&e.name);
            let mut style = Style::default().fg(color);
            if i == app.selected_index {
                style = style.bg(Color::Indexed(237)).add_modifier(Modifier::BOLD);
            }
            ListItem::new(Line::from(vec![
                Span::styled(icon, Style::default().fg(color)),
                Span::styled(name, style),
            ]))
        })
        .collect();

    f.render_widget(
        List::new(items).block(Block::default().borders(Borders::ALL)),
        body[0],
    );

    if app.show_preview {
        let p = Paragraph::new(app.preview_content.as_deref().unwrap_or("No content"))
            .block(Block::default().borders(Borders::ALL).title(" Preview "))
            .scroll((app.preview_scroll, 0))
            .wrap(Wrap { trim: false });
        f.render_widget(p, body[1]);
    }

    let help_menu = match app.input_mode {
        InputMode::Normal => {
            " [/] Search | [o] Preview | [h/j/k/l] Nav | [ENTER] Open | [Ctrl-c] Quit "
        }
        InputMode::Search => " [ESC] Normal Mode | [ENTER] Finish | [BACKSPACE] Delete ",
    };

    let footer_text = Line::from(vec![
        Span::styled(" focused: ", Style::default().add_modifier(Modifier::DIM)),
        Span::styled(
            if matches!(app.input_mode, InputMode::Search) {
                "Search Bar"
            } else {
                "File Explorer"
            },
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw(" | "),
        Span::styled(help_menu, Style::default().fg(Color::DarkGray)),
    ]);

    let footer = Paragraph::new(footer_text);
    f.render_widget(footer, chunks[2]);
}

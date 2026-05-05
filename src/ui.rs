use std::io;
use std::io::Stdout;

use ratatui::backend::CrosstermBackend;
use ratatui::layout::{Alignment, Constraint, Layout};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, List, ListItem, ListState, Paragraph};
use ratatui::Terminal;

use crate::app_state::AppState;
use crate::config::Config;

pub fn render(
    terminal: &mut Terminal<CrosstermBackend<Stdout>>,
    state: &AppState,
    config: &Config,
) -> io::Result<()> {
    terminal.draw(|frame| {
        let chunks = Layout::default()
            .constraints([
                Constraint::Length(1),
                Constraint::Min(0),
                Constraint::Length(3),
            ])
            .split(frame.area());

        let header = Paragraph::new(Span::styled(
            "User Profile Generator",
            Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
        ))
        .alignment(Alignment::Center);
        frame.render_widget(header, chunks[0]);

        let mut items: Vec<ListItem> = Vec::new();
        for (i, field) in config.fields.iter().enumerate() {
            let is_selected = i == state.selected_field;
            let copied = state.copied_fields.contains(&i);

            let prefix = if is_selected { "> " } else { "  " };
            let suffix = if copied { " ✓" } else { "" };

            let value = state.field_value(field);

            let line_style = if is_selected {
                Style::default()
                    .fg(Color::White)
                    .bg(Color::DarkGray)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::White)
            };

            let label_style = if is_selected {
                Style::default()
                    .fg(Color::Yellow)
                    .bg(Color::DarkGray)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::Green)
            };

            let copied_style = if is_selected {
                Style::default()
                    .fg(Color::Green)
                    .bg(Color::DarkGray)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::Green)
            };

            let line = Line::from(vec![
                Span::styled(prefix, line_style),
                Span::styled(format!("{:<12}", field.label()), label_style),
                Span::styled(": ", line_style),
                Span::styled(value.clone(), line_style),
                Span::styled(suffix, copied_style),
            ]);

            items.push(ListItem::new(line));
        }

        let list = List::new(items)
            .block(
                Block::default()
                    .title("Generated User Profile")
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::Cyan)),
            )
            .highlight_style(
                Style::default()
                    .fg(Color::White)
                    .bg(Color::DarkGray)
                    .add_modifier(Modifier::BOLD),
            )
            .highlight_symbol(" ");

        let mut list_state = ListState::default();
        list_state.select(Some(state.selected_field));
        frame.render_stateful_widget(list, chunks[1], &mut list_state);

        let status_color = if state.status_message.contains("Error") {
            Color::Red
        } else {
            Color::White
        };
        let status = Paragraph::new(Line::from(vec![
            Span::styled(
                "↓/j ↑/k: navigate  ",
                Style::default().fg(Color::DarkGray),
            ),
            Span::styled(
                "Enter/Space: copy  ",
                Style::default().fg(Color::DarkGray),
            ),
            Span::styled(
                "r: refresh  ",
                Style::default().fg(Color::DarkGray),
            ),
            Span::styled(
                "q: quit",
                Style::default().fg(Color::DarkGray),
            ),
        ]))
        .alignment(Alignment::Center);
        frame.render_widget(status, chunks[2]);

        if !state.status_message.is_empty() {
            let msg = Paragraph::new(Span::styled(
                &state.status_message,
                Style::default().fg(status_color),
            ))
            .alignment(Alignment::Center);
            let msg_area = chunks[1];
            use ratatui::layout::Rect;
            let msg_rect = Rect {
                x: msg_area.x + 2,
                y: msg_area.y + msg_area.height.saturating_sub(1),
                width: msg_area.width.saturating_sub(4),
                height: 1,
            };
            frame.render_widget(msg, msg_rect);
        }
    })?;
    Ok(())
}

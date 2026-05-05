mod api;
mod app_state;
mod clipboard_field;
mod config;
mod email;
mod models;
mod password;
mod terminal;
mod ui;

use arboard::Clipboard;
use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use std::io::{self, Stdout};
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;

use app_state::AppState;
use config::Config;
use terminal::{restore_terminal, setup_terminal};
use ui::render;

fn main() -> io::Result<()> {
    let config = Config::from_env();
    let mut state = AppState::new(&config);
    let mut terminal = setup_terminal()?;

    let result = run(&mut terminal, &mut state, &config);

    restore_terminal()?;

    match result {
        Ok(_) => println!("Exiting..."),
        Err(e) => eprintln!("Error: {}", e),
    }

    Ok(())
}

fn run(
    terminal: &mut Terminal<CrosstermBackend<Stdout>>,
    state: &mut AppState,
    config: &Config,
) -> io::Result<()> {
    let mut clipboard = Clipboard::new().expect("Failed to open clipboard");

    loop {
        render(terminal, state, config)?;

        if let Event::Key(key) = event::read()? {
            if key.kind != KeyEventKind::Press {
                continue;
            }

            match key.code {
                KeyCode::Char('q') | KeyCode::Esc => return Ok(()),
                KeyCode::Down | KeyCode::Char('j') => {
                    state.selected_field = (state.selected_field + 1).min(config.fields.len() - 1);
                }
                KeyCode::Up | KeyCode::Char('k') => {
                    state.selected_field = state.selected_field.saturating_sub(1);
                }
                KeyCode::Enter | KeyCode::Char(' ') => {
                    let field = &config.fields[state.selected_field];
                    let value = state.field_value(field);
                    clipboard
                        .set()
                        .text(&value)
                        .expect("Failed to set clipboard text");
                    state
                        .copied_fields
                        .insert(state.selected_field);
                    state.status_message = format!("Copied {} to clipboard", field.label());
                    state.selected_field = (state.selected_field + 1).min(config.fields.len() - 1);
                }
                KeyCode::Char('r') => {
                    state.refresh(config);
                }
                _ => {}
            }
        }
    }
}

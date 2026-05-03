use arboard::Clipboard;
use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use crossterm::terminal::{self, EnterAlternateScreen, LeaveAlternateScreen};
use crossterm::ExecutableCommand;
use rand::thread_rng;
use rand::Rng;
use ratatui::backend::CrosstermBackend;
use ratatui::layout::{Alignment, Constraint, Layout};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, List, ListItem, ListState, Paragraph};
use ratatui::Terminal;
use std::collections::HashSet;
use std::io::{self, Stdout};

// --- API Response Structures ---

#[derive(serde::Deserialize)]
struct ApiResponse {
    results: Vec<User>,
}

#[derive(serde::Deserialize)]
struct User {
    name: Name,
    email: String,
    login: Login,
}

#[derive(serde::Deserialize)]
struct Name {
    first: String,
    last: String,
}

#[derive(serde::Deserialize)]
struct Login {
    username: String,
    password: String,
}

// --- Clipboard Field Enum ---

#[derive(Debug, Clone)]
enum ClipboardField {
    Email,
    Password,
    First,
    Last,
    FullName,
    Username,
}

impl ClipboardField {
    fn from_str(s: &str) -> Option<Self> {
        match s.trim().to_lowercase().as_str() {
            "email" => Some(Self::Email),
            "password" => Some(Self::Password),
            "first" => Some(Self::First),
            "last" => Some(Self::Last),
            "fullname" => Some(Self::FullName),
            "username" => Some(Self::Username),
            _ => None,
        }
    }

    fn label(&self) -> &'static str {
        match self {
            Self::Email => "Email",
            Self::Password => "Password",
            Self::First => "First Name",
            Self::Last => "Last Name",
            Self::FullName => "Full Name",
            Self::Username => "Username",
        }
    }
}

// --- Configuration ---

struct Config {
    fields: Vec<ClipboardField>,
    password_min_length: usize,
    password_require_upper: bool,
    password_require_special: bool,
    password_require_digit: bool,
}

impl Config {
    fn from_env() -> Self {
        let fields_str =
            std::env::var("FIELDS").unwrap_or_else(|_| "email,password,first,last".into());
        let fields: Vec<ClipboardField> = fields_str
            .split(',')
            .filter_map(|s| {
                let field = ClipboardField::from_str(s);
                if field.is_none() {
                    eprintln!("Warning: unknown field '{}', skipping", s.trim());
                }
                field
            })
            .collect();

        if fields.is_empty() {
            panic!("No valid fields configured in FIELDS env var");
        }

        let password_min_length: usize = std::env::var("PASSWORD_MIN_LENGTH")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(8);

        let bool_env = |var: &str, default: bool| -> bool {
            std::env::var(var)
                .map(|v| v != "0" && v.to_lowercase() != "false")
                .unwrap_or(default)
        };

        Self {
            fields,
            password_min_length,
            password_require_upper: bool_env("PASSWORD_REQUIRE_UPPER", false),
            password_require_special: bool_env("PASSWORD_REQUIRE_SPECIAL", false),
            password_require_digit: bool_env("PASSWORD_REQUIRE_DIGIT", false),
        }
    }
}

// --- Email Modification ---

fn modify_email(email: &str, rng: &mut impl Rng) -> String {
    if let Some((local, domain)) = email.split_once('@') {
        if let Some(dot_pos) = domain.rfind('.') {
            let domain_name = &domain[..dot_pos];
            let tld = &domain[dot_pos..];
            let hash: String = (0..4)
                .map(|_| format!("{:x}", rng.gen_range(0..=15)))
                .collect();
            return format!("{}@{}{}{}", local, domain_name, hash, tld);
        }
    }

    email.to_string()
}

// --- Password Adjustment ---

fn adjust_password(password: &str, config: &Config) -> String {
    if !config.password_require_upper
        && !config.password_require_special
        && !config.password_require_digit
        && password.len() >= config.password_min_length
    {
        return password.to_string();
    }

    let mut pwd = password.to_string();
    let mut rng = thread_rng();
    let specials: Vec<char> = "!@#$%^&*()_+-=[]{}|;:,.<>?".chars().collect();

    let insert_random = |pwd: &mut String, rng: &mut rand::prelude::ThreadRng, ch: char| {
        let pos = rng.gen_range(0..=pwd.len());
        pwd.insert(pos, ch);
    };

    if config.password_require_upper && !pwd.chars().any(|c| c.is_ascii_uppercase()) {
        let upper = rng.gen_range(b'A'..=b'Z') as char;
        insert_random(&mut pwd, &mut rng, upper);
    }

    if config.password_require_special && !pwd.chars().any(|c| specials.contains(&c)) {
        let special = specials[rng.gen_range(0..specials.len())];
        insert_random(&mut pwd, &mut rng, special);
    }

    if config.password_require_digit && !pwd.chars().any(|c| c.is_ascii_digit()) {
        let digit = rng.gen_range('0'..='9');
        insert_random(&mut pwd, &mut rng, digit);
    }

    if pwd.len() < config.password_min_length {
        let needed = config.password_min_length - pwd.len();
        let chars: Vec<char> = "abcdefghijklmnopqrstuvwxyz0123456789!@#$%^&*"
            .chars()
            .collect();
        for _ in 0..needed {
            let pos = rng.gen_range(0..=pwd.len());
            pwd.insert(pos, chars[rng.gen_range(0..chars.len())]);
        }
    }

    pwd
}

// --- Fetch User ---

fn fetch_user() -> User {
    let client = reqwest::blocking::Client::new();
    let response = client
        .get("https://randomuser.me/api/")
        .send()
        .expect("Failed to fetch from randomuser.me");

    let api_response: ApiResponse = response.json().expect("Failed to parse API response");
    api_response
        .results
        .into_iter()
        .next()
        .expect("No user in response")
}

// --- App State ---

struct AppState {
    user: User,
    email: String,
    password: String,
    selected_field: usize,
    copied_fields: HashSet<usize>,
    status_message: String,
    loading: bool,
}

impl AppState {
    fn new(config: &Config) -> Self {
        let mut rng = thread_rng();
        let user = fetch_user();
        let email = modify_email(&user.email, &mut rng);
        let password = adjust_password(&user.login.password, config);
        Self {
            user,
            email,
            password,
            selected_field: 0,
            copied_fields: HashSet::new(),
            status_message: String::new(),
            loading: false,
        }
    }

    fn refresh(&mut self, config: &Config) {
        self.loading = true;
        let mut rng = thread_rng();
        self.user = fetch_user();
        self.email = modify_email(&self.user.email, &mut rng);
        self.password = adjust_password(&self.user.login.password, config);
        self.selected_field = 0;
        self.copied_fields.clear();
        self.loading = false;
        self.status_message = "New user generated!".into();
    }

    fn field_value(&self, field: &ClipboardField) -> String {
        match field {
            ClipboardField::Email => self.email.clone(),
            ClipboardField::Password => self.password.clone(),
            ClipboardField::First => self.user.name.first.clone(),
            ClipboardField::Last => self.user.name.last.clone(),
            ClipboardField::FullName => format!("{} {}", self.user.name.first, self.user.name.last),
            ClipboardField::Username => self.user.login.username.clone(),
        }
    }
}

// --- Rendering ---

fn render(
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

// --- Terminal Setup / Teardown ---

fn setup_terminal() -> io::Result<Terminal<CrosstermBackend<Stdout>>> {
    let mut stdout = io::stdout();
    stdout.execute(EnterAlternateScreen)?;
    terminal::enable_raw_mode()?;
    let backend = CrosstermBackend::new(stdout);
    Terminal::new(backend)
}

fn restore_terminal() -> io::Result<()> {
    let mut stdout = io::stdout();
    stdout.execute(LeaveAlternateScreen)?;
    terminal::disable_raw_mode()?;
    Ok(())
}

// --- Main ---

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

use arboard::Clipboard;
use clap::Parser;
use rand::thread_rng;
use rand::Rng;
use std::io;

// --- CLI Arguments ---

#[derive(Parser)]
#[command(
    name = "user_generator",
    about = "Generate random user profiles and copy fields to clipboard"
)]
struct Args {
    #[arg(short = 'd', long, default_value = "")]
    domain_append: String,
}

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
    password: String,
}

// --- Clipboard Field Enum ---

#[derive(Debug, Clone)]
enum ClipboardField {
    Email,
    Password,
    First,
    Last,
}

impl ClipboardField {
    fn from_str(s: &str) -> Option<Self> {
        match s.trim().to_lowercase().as_str() {
            "email" => Some(Self::Email),
            "password" => Some(Self::Password),
            "first" => Some(Self::First),
            "last" => Some(Self::Last),
            _ => None,
        }
    }

    fn label(&self) -> &'static str {
        match self {
            Self::Email => "Email",
            Self::Password => "Password",
            Self::First => "First Name",
            Self::Last => "Last Name",
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

fn modify_email(email: &str, append: &str) -> String {
    if append.is_empty() {
        return email.to_string();
    }

    if let Some((local, domain)) = email.split_once('@') {
        if let Some(dot_pos) = domain.rfind('.') {
            let domain_name = &domain[..dot_pos];
            let tld = &domain[dot_pos..];
            return format!("{}@{}{}{}", local, domain_name, append, tld);
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

// --- Display Profile ---

fn display_profile(user: &User, email: &str, password: &str) {
    let first_name = user.name.first.clone();
    let last_name = user.name.last.clone();
    let fields = [
        ("First Name", first_name),
        ("Last Name", last_name),
        ("Email", email.to_string()),
        ("Password", password.to_string()),
    ];

    let max_value_len = fields.iter().map(|(_, v)| v.len()).max().unwrap_or(0);
    let label_width = fields.iter().map(|(l, _)| l.len()).max().unwrap_or(0);
    let inner_width = label_width + 3 + max_value_len;
    let border_width = inner_width + 4;

    let top_bottom = format!("╔{}╗", "═".repeat(border_width));
    let separator = format!("╠{}╣", "═".repeat(border_width));
    let title = "Generated User Profile";
    let title_padding = (border_width - title.len()) / 2;
    let title_line = format!(
        "║{}{}{}║",
        " ".repeat(title_padding),
        title,
        " ".repeat(border_width - title_padding - title.len())
    );

    println!("\n{}", top_bottom);
    println!("{}", title_line);
    println!("{}", separator);
    for (label, value) in &fields {
        println!(
            "║  {:<label_width$} : {:<inner_value$}  ║",
            label,
            value,
            label_width = label_width,
            inner_value = inner_width - label_width - 3
        );
    }
    let bottom = top_bottom.replace('╔', "╚").replace('╗', "╝");
    println!("{}\n", bottom);
}

// --- Main ---

fn main() {
    let args = Args::parse();
    let config = Config::from_env();

    println!("User Profile Generator");
    println!("Domain append: '{}'", args.domain_append);
    println!(
        "Fields: {:?}",
        config.fields.iter().map(|f| f.label()).collect::<Vec<_>>()
    );
    if config.password_require_upper
        || config.password_require_special
        || config.password_require_digit
        || config.password_min_length > 0
    {
        print!(
            "Password restrictions: min_length={}",
            config.password_min_length
        );
        if config.password_require_upper {
            print!(", upper");
        }
        if config.password_require_special {
            print!(", special");
        }
        if config.password_require_digit {
            print!(", digit");
        }
        println!();
    }

    let mut clipboard = Clipboard::new().expect("Failed to open clipboard");

    loop {
        println!("\nFetching user from randomuser.me...");
        let user = fetch_user();

        let email = modify_email(&user.email, &args.domain_append);
        let password = adjust_password(&user.login.password, &config);

        display_profile(&user, &email, &password);

        println!("Select the terminal, then press Enter to begin clipboard insertion.");
        let mut input = String::new();
        io::stdin()
            .read_line(&mut input)
            .expect("Failed to read input");

        for (i, field) in config.fields.iter().enumerate() {
            let value = match field {
                ClipboardField::Email => email.clone(),
                ClipboardField::Password => password.clone(),
                ClipboardField::First => user.name.first.clone(),
                ClipboardField::Last => user.name.last.clone(),
            };

            println!("  Copying {}...", field.label());
            clipboard
                .set()
                .text(value.clone())
                .expect("Failed to set clipboard text");
            println!("    ✓ Copied: {}", value);

            if i < config.fields.len() - 1 {
                print!("  Press Enter to copy the next field...");
                io::stdin()
                    .read_line(&mut String::new())
                    .expect("Failed to read input");
            }
        }

        println!("\nAll fields copied to clipboard!");
        println!("Generate another user? (Y/n): ");
        let mut response = String::new();
        io::stdin()
            .read_line(&mut response)
            .expect("Failed to read input");

        if response.trim().to_lowercase() == "n" {
            println!("Exiting...");
            break;
        }
    }
}

use crate::clipboard_field::ClipboardField;

pub struct Config {
    pub fields: Vec<ClipboardField>,
    pub password_min_length: usize,
    pub password_require_upper: bool,
    pub password_require_special: bool,
    pub password_require_digit: bool,
}

impl Config {
    pub fn from_env() -> Self {
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

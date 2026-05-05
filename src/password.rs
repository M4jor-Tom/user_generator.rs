use crate::config::Config;
use rand::{thread_rng, Rng};

pub fn adjust_password(password: &str, config: &Config) -> String {
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

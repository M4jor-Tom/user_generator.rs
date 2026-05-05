use std::collections::HashSet;

use rand::thread_rng;

use crate::api;
use crate::clipboard_field::ClipboardField;
use crate::config::Config;
use crate::email;
use crate::models::User;
use crate::password;

pub struct AppState {
    pub user: User,
    pub email: String,
    pub password: String,
    pub selected_field: usize,
    pub copied_fields: HashSet<usize>,
    pub status_message: String,
    pub loading: bool,
}

impl AppState {
    pub fn new(config: &Config) -> Self {
        let mut rng = thread_rng();
        let user = api::fetch_user();
        let email = email::modify_email(&user.email, &mut rng);
        let password = password::adjust_password(&user.login.password, config);
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

    pub fn refresh(&mut self, config: &Config) {
        self.loading = true;
        let mut rng = thread_rng();
        self.user = api::fetch_user();
        self.email = email::modify_email(&self.user.email, &mut rng);
        self.password = password::adjust_password(&self.user.login.password, config);
        self.selected_field = 0;
        self.copied_fields.clear();
        self.loading = false;
        self.status_message = "New user generated!".into();
    }

    pub fn field_value(&self, field: &ClipboardField) -> String {
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

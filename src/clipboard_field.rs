#[derive(Debug, Clone)]
pub enum ClipboardField {
    Email,
    Password,
    First,
    Last,
    FullName,
    Username,
}

impl ClipboardField {
    pub fn from_str(s: &str) -> Option<Self> {
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

    pub fn label(&self) -> &'static str {
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

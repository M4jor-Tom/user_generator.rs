#[derive(serde::Deserialize)]
pub struct ApiResponse {
    pub results: Vec<User>,
}

#[derive(serde::Deserialize)]
pub struct User {
    pub name: Name,
    pub email: String,
    pub login: Login,
}

#[derive(serde::Deserialize)]
pub struct Name {
    pub first: String,
    pub last: String,
}

#[derive(serde::Deserialize)]
pub struct Login {
    pub username: String,
    pub password: String,
}

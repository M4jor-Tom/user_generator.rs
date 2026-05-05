use crate::models::{ApiResponse, User};
use reqwest::blocking::Client;

pub fn fetch_user() -> User {
    let client = Client::new();
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

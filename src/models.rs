use serde::{Deserialize, Serialize};
use std::error::Error;

#[derive(Serialize)]
pub struct User<'a> {
    pub id: Option<i32>,
    pub username: &'a str,
    pub pwd: &'a str,
}

#[derive(Deserialize)]
pub struct LoginPayload<'a> {
    pub username: &'a str,
    pub pwd: &'a str,
}

#[derive(Debug)]
// Struct to represent the result of your query
pub struct UserPassword {
    pub pwd: String,
}

// When a new User instance is created, the references to 'username' and 'pwd'
// Will be tied to the lifetimes of the strings we pass to the 'new' method.
// This provides more flexibility and avoids forcing the strings to have a 'static lifetime.
impl<'a> User<'a> {
    pub fn new(
        id: Option<i32>,
        username: &'a str,
        pwd: &'a str,
    ) -> Result<User<'a>, Box<dyn Error>> {
        Ok(User { id, username, pwd })
    }
}

impl<'a> LoginPayload<'a> {
    pub fn new(username: &'a str, pwd: &'a str) -> Result<LoginPayload<'a>, Box<dyn Error + Send>> {
        Ok(LoginPayload { username, pwd })
    }
}
use std::error::Error;

use crate::identity::{AccessResponse, IdentityStore, Outcome};
use serde_derive::Deserialize;
use std::fs::File;
use std::path::Path;

#[derive(Deserialize)]
pub struct JsonIdentitySettings {
    pub file_path: String,
}

#[derive(Deserialize)]
struct User {
    username: String,
    token: String,
    access: bool,
}

pub struct Json {
    settings: JsonIdentitySettings,
}

impl Json {
    pub fn new(settings: JsonIdentitySettings) -> Self {
        Self { settings }
    }
}

#[async_trait]
impl IdentityStore for Json {
    async fn access(&mut self, token: &str) -> Result<AccessResponse, Box<dyn Error>> {
        let json_file_path = Path::new(&self.settings.file_path);
        let file = File::open(json_file_path)?;
        let users: Vec<User> = serde_json::from_reader(file)?;

        for user in users {
            if user.token == token {
                return Ok(AccessResponse {
                    outcome: if user.access {
                        Outcome::Success
                    } else {
                        Outcome::Revoked
                    },
                    name: Some(user.username),
                });
            }
        }

        return Ok(AccessResponse {
            outcome: Outcome::Unknown,
            name: None,
        });
    }
}

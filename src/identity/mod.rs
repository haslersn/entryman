pub mod ldap;

use std::error::Error;

use serde_derive::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Copy, FromFormField, PartialEq)]
pub enum Outcome {
    Success,
    Revoked,
    Unknown,
}

#[derive(Serialize, Deserialize, Clone, PartialEq)]
pub struct AccessResponse {
    pub outcome: Outcome,
    pub name: Option<String>,
}

pub trait IdentityStore: Send {
    fn access(&mut self, token: &str) -> Result<AccessResponse, Box<dyn Error>>;
}

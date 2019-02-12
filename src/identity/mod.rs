pub mod ldap;

use crate::util::Result;

#[derive(Serialize, Deserialize, Clone, Copy, PartialEq)]
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
    fn access(&mut self, token: &str) -> Result<AccessResponse>;
}

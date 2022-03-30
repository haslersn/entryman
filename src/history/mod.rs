pub mod json_history;

use std::error::Error;

use crate::identity::{AccessResponse, Outcome};
use serde_derive::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone)]
pub struct HistoryEntry {
    pub time: u64,
    pub token: String,
    pub response: AccessResponse,
}

pub trait History: Send {
    fn insert(&mut self, entry: HistoryEntry) -> Result<(), Box<dyn Error>>;

    fn query(
        &self,
        time_min: Option<u64>,
        time_max: Option<u64>,
        token: Option<&str>,
        name: Option<&str>,
        outcome: Option<Outcome>,
        only_latest: bool,
    ) -> Result<Vec<HistoryEntry>, Box<dyn Error>>;
}

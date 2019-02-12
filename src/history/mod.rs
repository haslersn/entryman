pub mod json_history;

use crate::identity::AccessResponse;
use crate::identity::Outcome;
use crate::util::Result;

#[derive(Serialize, Deserialize, Clone)]
pub struct HistoryEntry {
    pub time: u64,
    pub token: String,
    pub response: AccessResponse,
}

pub trait History: Send {
    fn insert(&mut self, entry: HistoryEntry) -> Result<()>;

    fn query(
        &self,
        time_min: Option<u64>,
        time_max: Option<u64>,
        token: Option<&str>,
        name: Option<&str>,
        outcome: Option<Outcome>,
        only_latest: bool,
    ) -> Result<Vec<HistoryEntry>>;
}

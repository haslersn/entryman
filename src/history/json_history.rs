use crate::history::History;
use crate::history::HistoryEntry;
use crate::identity::Outcome;
use crate::util::Result;
use std::fs::File;
use std::fs::OpenOptions;
use std::io::BufRead;
use std::io::BufReader;
use std::io::BufWriter;
use std::io::Write;

#[derive(Deserialize)]
pub struct JsonHistorySettings {
    pub filename: String,
}

pub struct JsonHistory {
    file: File,
    history: Vec<HistoryEntry>,
}

impl JsonHistory {
    pub fn new(settings: JsonHistorySettings) -> Result<Self> {
        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(&settings.filename)?;
        let history = history_read(&file)?;
        Ok(Self { file, history })
    }
}

impl History for JsonHistory {
    fn insert(&mut self, entry: HistoryEntry) -> Result<()> {
        history_append(&mut self.file, &entry)?;
        self.history.push(entry);
        Ok(())
    }

    fn query(
        &self,
        time_min: Option<u64>,
        time_max: Option<u64>,
        token: Option<&str>,
        name: Option<&str>,
        outcome: Option<Outcome>,
        only_latest: bool,
    ) -> Result<Vec<HistoryEntry>> {
        let iter = self.history.iter().filter(|history_entry| {
            if time_min.unwrap_or(0) > history_entry.time {
                return false;
            }
            if time_max.unwrap_or(u64::max_value()) < history_entry.time {
                return false;
            }
            if token.map_or(false, |v| v != history_entry.token) {
                return false;
            }
            if outcome
                .as_ref()
                .map_or(false, |v| *v != history_entry.response.outcome)
            {
                return false;
            }
            if name.is_some() && name != history_entry.response.name.as_ref().map(|x| &**x) {
                return false;
            }
            true
        });
        Ok(if only_latest {
            iter.rev().take(1).map(|x| x.clone()).collect()
        } else {
            iter.map(|x| x.clone()).collect()
        })
    }
}

fn history_append(file: &mut File, entry: &HistoryEntry) -> Result<()> {
    let mut writer = BufWriter::new(file);
    serde_json::to_writer(&mut writer, entry)?;
    writer.write("\n".as_bytes())?;
    Ok(())
}

fn history_read(file: &File) -> Result<Vec<HistoryEntry>> {
    BufReader::new(file)
        .lines()
        .map(|line| Ok(serde_json::from_str(&(line?))?))
        .collect()
}

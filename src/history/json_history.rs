use crate::history::{History, HistoryEntry};
use crate::identity::Outcome;
use serde_derive::Deserialize;
use std::error::Error;
use std::fs::{File, OpenOptions};
use std::io::{BufRead, BufReader, BufWriter, Write};

#[derive(Deserialize)]
pub struct JsonHistorySettings {
    pub filename: String,
}

pub struct JsonHistory {
    settings: JsonHistorySettings,
    file: File,
}

impl JsonHistory {
    pub fn new(settings: JsonHistorySettings) -> Result<Self, Box<dyn Error>> {
        let file = OpenOptions::new()
            .write(true)
            .create(true)
            .open(&settings.filename)?;
        Ok(Self { settings, file })
    }
}

impl History for JsonHistory {
    fn insert(&mut self, entry: HistoryEntry) -> Result<(), Box<dyn Error>> {
        let mut writer = BufWriter::new(&mut self.file);
        serde_json::to_writer(&mut writer, &entry)?;
        writer.write("\n".as_bytes())?;
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
    ) -> Result<Vec<HistoryEntry>, Box<dyn Error>> {
        let reader = BufReader::new(
            OpenOptions::new()
                .read(true)
                .open(&self.settings.filename)?,
        );
        let iter = reader.lines().enumerate().filter_map(|(n, line)| {
            let location = format!("{}:{}", self.settings.filename, n + 1);
            match line {
                Err(e) => {
                    warn!("Failed to read history entry at {}: {:?}", location, e);
                    None
                }
                Ok(line) => match serde_json::from_str::<HistoryEntry>(&line) {
                    Err(e) => {
                        warn!(
                            "Failed to deserialize history entry at {}: {:?}",
                            location, e
                        );
                        None
                    }
                    Ok(history_entry) => {
                        if time_min.unwrap_or(0) > history_entry.time {
                            return None;
                        }
                        if time_max.unwrap_or(u64::max_value()) < history_entry.time {
                            return None;
                        }
                        if token.map_or(false, |v| v != history_entry.token) {
                            return None;
                        }
                        if outcome
                            .as_ref()
                            .map_or(false, |v| *v != history_entry.response.outcome)
                        {
                            return None;
                        }
                        if name.is_some()
                            && name != history_entry.response.name.as_ref().map(|x| &**x)
                        {
                            return None;
                        }
                        Some(history_entry)
                    }
                },
            }
        });
        Ok(if only_latest {
            iter.last().into_iter().collect()
        } else {
            iter.collect()
        })
    }
}

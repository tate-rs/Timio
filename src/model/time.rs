use chrono::{DateTime, Local};
use serde::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RunningEntry {
    pub name: String,
    pub description: Option<String>,
    pub start: DateTime<Local>,
}

impl RunningEntry {
    pub fn finish(&self, stop: DateTime<Local>) -> FinishedEntry {
        FinishedEntry { 
            name: self.name.clone(), 
            description: self.description.clone(),
            start: self.start, 
            stop,
            duration: (stop.timestamp() - self.start.timestamp()),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FinishedEntry {
    pub name: String,
    pub description: Option<String>,
    pub start: DateTime<Local>,
    pub stop: DateTime<Local>,
    pub duration: i64,
}

impl FinishedEntry {
    pub fn elapsed(&self) -> chrono::TimeDelta {
        self.stop - self.start
    }
}
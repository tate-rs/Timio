use std::collections::{BTreeMap, HashMap};

use crate::model::{FinishedEntry, RunningEntry};
use super::error::TrackerError;
use chrono::{Local, NaiveDate};
use crate::store::Persistence;

#[derive(Clone)]
pub struct TimeTracker {
    running: HashMap<String, RunningEntry>,    
    uncommited_running: HashMap<String, RunningEntry>,
    uncommited_finished: HashMap<String, FinishedEntry>,
}

impl Default for TimeTracker {
    fn default() -> Self {
        Self::new()
    }
}

impl TimeTracker {
    pub fn new() -> Self {
        Self {
            running: HashMap::new(),
            uncommited_running: HashMap::new(),
            uncommited_finished: HashMap::new()
        }
    }

    pub fn get_running(&self) -> HashMap<String, RunningEntry> {
        self.running.clone()
    }

    pub async fn start(&mut self, name: impl ToString, description: Option<String>) -> Result<(), TrackerError> {
        let name = name.to_string();

        if self.running.contains_key(&name) {
            return Err(TrackerError::AlreadyTracked(name));
        }

        let entry = RunningEntry {
            name: name.clone(),
            description,
            start: Local::now() ,
        };

        self.running.insert(name.clone(), entry.clone());
        self.uncommited_running.insert(name, entry);

        Ok(())
    }

    pub async fn stop(&mut self, name: impl ToString) -> Result<FinishedEntry, TrackerError> {
        let name = name.to_string();

        // TODO split time in between span of days 
        //  -> working on something that overlaps with the next day - split it into two days.

        if let Some(running) = self.running.remove(&name) {
            let finished = running.finish(Local::now());
            self.uncommited_finished.insert(name, finished.clone());
            Ok(finished)
        }
        else {
            Err(TrackerError::NotTracking(name))
        }
    }

    pub async fn stop_all(&mut self) -> Vec<FinishedEntry> {
        let now = Local::now();

        self.running.drain().map(|(name, entry)| {
            let finished = entry.finish(now);
            self.uncommited_finished.insert(name, finished.clone());
            finished
        }).collect()
    }

    pub async fn list_monthly(&self, from: Option<NaiveDate>, to: Option<NaiveDate>, persistence: &dyn Persistence) -> BTreeMap<NaiveDate, FinishedEntry> {
        let map = BTreeMap::new();

        map
    }

    pub async fn load(&mut self, persistence: &dyn Persistence) {
        persistence.running_list()
            .await.unwrap()
            .iter().for_each(|entry| {
                self.running.insert(entry.name.clone(), entry.clone());
            });
        }

    pub async fn commit(&mut self, persistence: &dyn Persistence) {
        for (name, entry) in &self.uncommited_finished {
            persistence.stop(name.clone(), Some(entry.stop.timestamp())).await.unwrap();
        }

        for (name, entry) in &self.uncommited_running {
            persistence.start(name.clone(), entry.description.clone(), Some(entry.start.timestamp())).await.unwrap();
        }

        self.uncommited_running.clear();
        self.uncommited_finished.clear();
    }
}

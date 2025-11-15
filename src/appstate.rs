use crate::{store, tracker::TimeTracker};

#[derive(Clone)]
pub struct AppState {
    pub tracker: TimeTracker,
    pub sqlite_store: store::SqliteStore,
}
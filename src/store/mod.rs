pub mod error;

use std::collections::BTreeMap;

use async_trait::async_trait;
use chrono::NaiveDate;
use crate::model::{FinishedEntry, RunningEntry};
pub use error::StoreError;

pub type Result<T> = std::result::Result<T, StoreError>;


#[async_trait]
pub trait Persistence: Send + Sync {
    /// Run migrations, set pragmas, etc.
    async fn init(&self) -> Result<()>;

    async fn running_list(&self) -> Result<Vec<RunningEntry>>;
    async fn finished_list(&self, from: Option<NaiveDate>, to: Option<NaiveDate>) -> Result<Vec<FinishedEntry>>;

    /// Start tracking; implicitly creates the project if it does not exist.
    async fn start(&self, project: String, description: Option<String>, started_at: Option<i64>) -> Result<()>;

    /// Stop the open entry for the given project.
    async fn stop(&self, project: String, ended_at: Option<i64>) -> Result<()>;
}

pub mod sqlite;
pub use sqlite::SqliteStore;
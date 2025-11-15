use thiserror::Error;

#[derive(Error, Debug)]
pub enum TrackerError {
    #[error("Task '{0}' is being already tracked")]
    AlreadyTracked(String),
    #[error("Task '{0}' is not being currently tracked")]
    NotTracking(String),
}
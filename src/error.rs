use thiserror::Error;


#[derive(Error, Debug)]
pub enum AppError {
    #[error("{0}")]
    Tracker(#[from] crate::tracker::TrackerError),

    #[error("{0}")]
    Store(#[from] crate::store::StoreError),
}
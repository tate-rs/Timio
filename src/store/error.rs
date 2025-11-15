use thiserror::Error;


#[derive(Debug, Error)]
pub enum StoreError {
    #[error("sql error: {0}")]
    Sqlx(#[from] sqlx::Error),

    #[error("sql error: {0}")]
    SqlxMigrate(#[from] sqlx::migrate::MigrateError),

    #[error("project not found: {0}")]
    ProjectNotFound(String),

    #[error("no open timer for project: {0}")]
    NoOpenTimer(String),

    #[error("constraint violated: {0}")]
    Constraint(String),

    #[error("{0}")]
    Other(String),
}
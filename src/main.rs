pub mod cli;
pub mod tracker;
pub mod error;
pub mod store;
pub mod appstate;
pub mod model;
pub mod utils;

use std::env;
use crate::store::Persistence;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let url = env::var("TIMIO_DB_PATH")
                        .unwrap_or("./timio.sqlite".to_string());

    let sqlite_store = store::SqliteStore::new(&url).await?;
    sqlite_store.init().await?;

    let mut app_state = appstate::AppState {
        tracker: tracker::TimeTracker::new(),
        sqlite_store,
    };

    if let Err(err) = cli::run(&mut app_state).await {
        println!("Error: {err}");
    }

    Ok(())
}

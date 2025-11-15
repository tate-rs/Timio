use std::collections::BTreeMap;
use std::str::FromStr;

use crate::model::{FinishedEntry, RunningEntry};
use crate::store::{Persistence, Result, StoreError};
use async_trait::async_trait;
use chrono::{DateTime, Local, NaiveDate, NaiveTime};
use sqlx::{Execute, Row, SqlitePool};
use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};
use sqlx::migrate::Migrator;

static MIGRATOR: Migrator = sqlx::migrate!("./migrations");

#[derive(Clone)]
pub struct SqliteStore {
    pool: SqlitePool,
}

impl SqliteStore {
    pub async fn new(path: &str) -> Result<Self> {
        let options = SqliteConnectOptions::from_str(path)?
            .create_if_missing(true)
            .foreign_keys(true);
            
        let pool = SqlitePoolOptions::new()
            .max_connections(1)
            .connect_with(options)
            .await?;

        Self::ensure_pragmas(&pool).await?;

        Ok(Self { pool })
    }

    async fn ensure_pragmas(pool: &SqlitePool) -> Result<()> {
        sqlx::query("PRAGMA foreign_keys = ON").execute(pool).await?;
        sqlx::query("PRAGMA busy_timeout = 5000").execute(pool).await?;
        Ok(())
    }

    async fn ensure_project_id(&self, name: &str) -> Result<i64> {
        sqlx::query("INSERT OR IGNORE INTO projects(name) VALUES (?)")
            .bind(name)
            .execute(&self.pool)
            .await?;

        let row = sqlx::query("SELECT id FROM projects WHERE name = ?")
            .bind(name)
            .fetch_one(&self.pool)
            .await
            .map_err(|_| StoreError::ProjectNotFound(name.to_string()))?;

        Ok(row.get::<i64, _>("id"))
    }

    async fn project_id_by_name(&self, name: &str) -> Result<i64> {
        let row = sqlx::query("SELECT id FROM projects WHERE name = ?")
            .bind(name)
            .fetch_one(&self.pool)
            .await
            .map_err(|_| StoreError::ProjectNotFound(name.to_string()))?;
        
        Ok(row.get::<i64, _>("id"))
    }

    fn now() -> DateTime<Local> {
        chrono::Local::now()
    }
}

#[async_trait]
impl Persistence for SqliteStore {
    async fn init(&self) -> Result<()> {
        MIGRATOR.run(&self.pool).await?;
        Ok(())
    }

    async fn running_list(&self) -> Result<Vec<RunningEntry>> {
        let rows = sqlx::query(
            r#"
            SELECT id, project_name, started_at, ended_at, description,
                   effective_duration_seconds
            FROM v_time_entries
            WHERE ended_at IS NULL 
            ORDER BY started_at DESC
            "#,
        )
        .fetch_all(&self.pool)
        .await?;


        let entries: Vec<RunningEntry> = rows
            .into_iter()
            .map(|row| {
                let start_utc = DateTime::from_timestamp_secs(row.get::<i64, _>("started_at")).unwrap_or_default();
                
                RunningEntry {
                    name: row.get::<String, _>("project_name"),
                    description: row.try_get::<String, _>("description").ok(),
                    start: start_utc.with_timezone(&Local),
                }
            })
            .collect();

        Ok(entries)
    }

    async fn finished_list(&self, from: Option<NaiveDate>, to: Option<NaiveDate>) -> Result<Vec<FinishedEntry>> {
        let from = from.unwrap_or(NaiveDate::MIN).and_time(NaiveTime::MIN);
        let to = to.unwrap_or(NaiveDate::MAX).and_time(NaiveTime::MIN);

        let rows = sqlx::query(
            r#"
            SELECT id, project_name, started_at, ended_at, description,
                   effective_duration_seconds
            FROM v_time_entries
            WHERE ended_at IS NOT NULL AND started_at > ? AND started_at < ?
            ORDER BY started_at DESC
            "#,
        )
        .bind(from.and_utc().timestamp())
        .bind(to.and_utc().timestamp())
        .fetch_all(&self.pool)
        .await?;

        let entries: Vec<FinishedEntry> = rows
            .into_iter()
            .map(|row| {
                let start_utc = DateTime::from_timestamp_secs(row.get::<i64, _>("started_at")).unwrap_or_default();
                let stop_utc = DateTime::from_timestamp_secs(row.get::<i64, _>("ended_at")).unwrap_or_default();

                FinishedEntry {
                    name: row.get::<String, _>("project_name"),
                    description: row.try_get::<String, _>("description").ok(),
                    start: start_utc.with_timezone(&Local),
                    stop: stop_utc.with_timezone(&Local),
                    duration: row.get::<i64, _>("effective_duration_seconds"),
                }
            })
            .collect();

        Ok(entries)
    }

    async fn monthly_finished_list(&self, from: Option<NaiveDate>, to: Option<NaiveDate>) -> BTreeMap<NaiveDate, FinishedEntry> {
        todo!()
    }

    async fn start(&self, project: String, description: Option<String>, started_at: Option<i64>) -> Result<()> {
        let proj_id = self.ensure_project_id(&project).await?;
        let now = started_at.unwrap_or_else(|| Self::now().timestamp());

        let res = sqlx::query(
            "INSERT INTO time_entries(project_id, started_at, ended_at, description)
             VALUES (?, ?, NULL, ?)",
        )
        .bind(proj_id)
        .bind(now)
        .bind(description)
        .execute(&self.pool)
        .await;

        match res {
            Ok(_) => Ok(()),
            Err(e) => {
                if let sqlx::Error::Database(db) = &e {
                    if db.message().contains("UNIQUE") {
                        return Err(StoreError::Constraint(
                            "project already has an open timer".into(),
                        ));
                    }

                    return Err(StoreError::from(e));
                }
                Err(StoreError::from(e))
            }
        }
    }

    async fn stop(&self, project: String, ended_at: Option<i64>) -> Result<()> {
        let proj_id = self.project_id_by_name(&project).await?;
        let now = ended_at.unwrap_or_else(|| Self::now().timestamp());

        let res = sqlx::query(
            "UPDATE time_entries
             SET ended_at = ?
             WHERE project_id = ? AND ended_at IS NULL",
        )
        .bind(now)
        .bind(proj_id)
        .execute(&self.pool)
        .await?;

        if res.rows_affected() == 0 {
            return Err(StoreError::NoOpenTimer(project.to_string()));
        }
        Ok(())
    }
}

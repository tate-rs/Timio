use chrono::{Local, NaiveDate};
use clap::{Parser, Subcommand};
use crate::error::AppError;
use crate::appstate::AppState;
use crate::store::Persistence;
use crate::utils;

#[derive(Parser)]
#[command(version, about, long_about = None)]
#[command(propagate_version = true)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Clone, Debug, clap::ValueEnum)]
enum ReportType {
    Daily,
    Monthly,
    Project,
}

#[derive(Subcommand)]
enum Commands {
    /// Starts tracking task by name
    Start { 
        name: String,
        description: Option<String>,
        #[arg(short, long)]
        concurrent: bool,
    },
    /// Stops tracking task by name
    Stop { 
        name: Option<String> 
    },
    /// Status of currently tracked task
    Status,
    /// Report of all tracked tasks
    Report {
        #[arg(short, long)]
        from: Option<NaiveDate>,
        #[arg(short, long)]
        to: Option<NaiveDate>,
        #[arg(short, long, value_enum, default_value_t=ReportType::Monthly)]
        report_type: ReportType
    },
    Purge,
}

pub async fn run(app_state: &mut AppState) -> Result<(), AppError> {

    let cli = Cli::parse();

    app_state.tracker.load(&app_state.sqlite_store).await;

    match cli.command {
        Commands::Start { name, description, concurrent } => {
            if !concurrent {
                let finished_tasks = app_state.tracker.stop_all().await;
                let finished_len = finished_tasks.len();

                if finished_len > 0 {
                    if finished_len == 1 {
                        println!("[Warning] Stopped task '{}'", finished_tasks[0].name);
                    }
                    else {
                        println!("[Warning] Stopped {finished_len} tasks");
                    }
                }
            }

            app_state.tracker.start(name.clone(), description).await?;
            println!("Started tracking task '{}'", name);
        },
        Commands::Stop { name } => {
            if let Some(name) = name {
                let finished = app_state.tracker.stop(name).await?;
                println!("Stopped task '{}'", finished.name);
            }
            else {
                let finished = app_state.tracker.stop_all().await;
                let finished_len = finished.len();

                match finished_len {
                    0 => println!("No tasks are running"),
                    1 => println!("Stopped task '{}'", finished[0].name),
                    _ => println!("Stopped {finished_len} tasks")
                }

            }
        },
        Commands::Status => {
            let running = app_state.tracker.get_running();

            if running.is_empty() {
                println!("No tasks are running");
            }
            else {
                println!("Running tasks:");

                for t in running.values() {
                    let current = t.finish(Local::now());
                    println!(" ❯ {} - {}", current.name, utils::time::duration_str_dynamic(current.duration));
                }
            }
        },
        Commands::Report { from , to, report_type } => {
            let r = app_state.sqlite_store.finished_list(from, to).await;
            println!("{r:#?}");
        },
        Commands::Purge => {
            todo!()
        },
    }

    app_state.tracker.commit(&app_state.sqlite_store).await;

    Ok(())
}
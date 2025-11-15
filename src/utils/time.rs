pub use chrono::Local;

pub fn duration_str(seconds: i64) -> String {
    let hrs = seconds / 3600;
    let mins = (seconds % 3600) / 60;
    let secs = seconds % 60;

    format!("{hrs:02}h:{mins:02}m:{secs:02}s")
}

pub fn duration_str_dynamic(seconds: i64) -> String {
    let hrs = seconds / 3600;
    let mins = (seconds % 3600) / 60;
    let secs = seconds % 60;

    let mut formatted = String::new();

    if hrs > 0 {
        formatted = format!("{hrs:02}h:");
    }

    if mins > 0 {
        formatted = format!("{formatted}{mins:02}m:");
    }

    format!("{formatted}{secs:02}s")
}
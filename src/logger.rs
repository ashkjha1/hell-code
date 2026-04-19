use std::fs::OpenOptions;
use std::io::Write;
use chrono::Local;

pub fn log_event(level: &str, message: &str) {
    let log_dir = ".hell-code";
    let log_path = format!("{}/app.log", log_dir);
    
    // Ensure directory exists
    let _ = std::fs::create_dir_all(log_dir);

    if let Ok(mut file) = OpenOptions::new()
        .create(true)
        .append(true)
        .open(&log_path)
    {
        let timestamp = Local::now().format("%Y-%m-%d %H:%M:%S");
        let _ = writeln!(file, "[{}] [{}] {}", timestamp, level, message);
    }
}

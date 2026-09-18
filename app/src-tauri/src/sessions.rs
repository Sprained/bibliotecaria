use serde::Serialize;
use std::io::Write;
use tauri::{AppHandle, Manager};

const SESSIONS_LOG: &str = "sessions.log";

#[derive(Serialize)]
#[serde(tag = "type")]
pub enum SessionEvent {
    AgentRun {
        mode: &'static str,
        note: Option<String>,
        success: bool,
        summary: String,
    },
    Decision {
        path: String,
        decision: &'static str,
    },
}

#[derive(Serialize)]
struct LogLine<'a> {
    timestamp: u64,
    #[serde(flatten)]
    event: &'a SessionEvent,
}

pub fn log_event(app: &AppHandle, event: SessionEvent) -> Result<(), String> {
    let path = app
        .path()
        .app_data_dir()
        .map_err(|e| e.to_string())?
        .join(SESSIONS_LOG);

    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_err(|e| e.to_string())?
        .as_secs();

    let line = serde_json::to_string(&LogLine {
        timestamp,
        event: &event,
    })
    .map_err(|e| e.to_string())?;

    let mut file = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&path)
        .map_err(|e| e.to_string())?;

    writeln!(file, "{line}").map_err(|e| e.to_string())
}

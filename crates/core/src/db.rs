#![cfg(feature = "sqlite-log")]

use crate::log_entry::LogEvent;
use rusqlite::{params, Connection, Result};
use std::path::PathBuf;

fn get_db_path() -> Result<PathBuf, String> {
    let data_dir = dirs::data_dir().ok_or("Could not find data directory")?;
    let db_dir = data_dir.join("rambo");
    std::fs::create_dir_all(&db_dir).map_err(|e| format!("Could not create db directory: {}", e))?;
    Ok(db_dir.join("rambo.db"))
}

pub fn init_db() -> Result<Connection, rusqlite::Error> {
    let path = get_db_path().map_err(|e| rusqlite::Error::InvalidPath(PathBuf::from(e)))?;
    let conn = Connection::open(path)?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS events (
            ts TEXT PRIMARY KEY,
            action TEXT,
            before_json TEXT,
            after_json TEXT,
            delta_mb INTEGER,
            pressure TEXT,
            details_json TEXT
        )",
        [],
    )?;

    Ok(conn)
}

pub fn log_event_sqlite(conn: &Connection, event: &LogEvent) -> Result<()> {
    let before_json = serde_json::to_string(&event.before).unwrap_or_else(|_| "null".to_string());
    let after_json = serde_json::to_string(&event.after).unwrap_or_else(|_| "null".to_string());
    let details_json = serde_json::to_string(&event.details).unwrap_or_else(|_| "null".to_string());
    let pressure = format!("{:?}", event.pressure);

    conn.execute(
        "INSERT INTO events (ts, action, before_json, after_json, delta_mb, pressure, details_json) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        params![
            event.ts,
            event.action,
            before_json,
            after_json,
            event.delta_mb,
            pressure,
            details_json
        ],
    )?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{PressureLevel};
    use chrono::Utc;

    #[test]
    fn can_init_db() {
        let conn = init_db();
        assert!(conn.is_ok());
    }

    #[test]
    fn can_log_event_sqlite() {
        let conn = init_db().unwrap();
        let event = LogEvent {
            ts: Utc::now().to_rfc3339(),
            action: "test_sqlite".to_string(),
            before: None,
            after: None,
            delta_mb: 0,
            pressure: PressureLevel::Normal,
            details: serde_json::json!({ "test": "data" }),
        };

        let result = log_event_sqlite(&conn, &event);
        assert!(result.is_ok());

        // Verify the entry was inserted
        let mut stmt = conn.prepare("SELECT action FROM events WHERE action = ?1").unwrap();
        let mut rows = stmt.query(params!["test_sqlite"]).unwrap();
        assert!(rows.next().unwrap().is_some());
    }
}
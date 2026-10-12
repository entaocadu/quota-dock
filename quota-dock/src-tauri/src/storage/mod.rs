use std::path::PathBuf;
use rusqlite::{params, Connection};
use crate::contracts::{PublicSettings, QuotaSnapshot};

pub fn get_app_db_path() -> PathBuf {
    if let Ok(appdata) = std::env::var("APPDATA") {
        let dir = PathBuf::from(appdata).join("QuotaDock");
        let _ = std::fs::create_dir_all(&dir);
        dir.join("quotadock.db")
    } else {
        PathBuf::from("quotadock.db")
    }
}

pub fn init_db(conn: &Connection) -> Result<(), rusqlite::Error> {
    conn.execute_batch(
        r#"
        CREATE TABLE IF NOT EXISTS snapshots (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            provider TEXT NOT NULL,
            origin TEXT NOT NULL,
            account_key TEXT NOT NULL,
            status TEXT NOT NULL,
            observed_at TEXT,
            attempted_at TEXT NOT NULL,
            ttl_until TEXT,
            json_sanitized TEXT NOT NULL
        );
        CREATE INDEX IF NOT EXISTS idx_snapshots_account_time ON snapshots(account_key, attempted_at DESC);

        CREATE TABLE IF NOT EXISTS settings (
            key TEXT PRIMARY KEY,
            value TEXT NOT NULL
        );
        "#,
    )?;
    Ok(())
}

pub fn save_snapshot(conn: &Connection, snapshot: &QuotaSnapshot) -> Result<(), rusqlite::Error> {
    let provider_str = serde_json::to_string(&snapshot.provider_id).unwrap_or_default();
    let origin_str = serde_json::to_string(&snapshot.origin).unwrap_or_default();
    let status_str = serde_json::to_string(&snapshot.status).unwrap_or_default();
    let json_sanitized = serde_json::to_string(snapshot).unwrap_or_default();

    conn.execute(
        r#"
        INSERT INTO snapshots (provider, origin, account_key, status, observed_at, attempted_at, ttl_until, json_sanitized)
        VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
        "#,
        params![
            provider_str,
            origin_str,
            snapshot.account_key,
            status_str,
            snapshot.observed_at,
            snapshot.attempted_at,
            snapshot.expires_at,
            json_sanitized
        ],
    )?;
    Ok(())
}

pub fn get_history(conn: &Connection, limit: usize) -> Result<Vec<QuotaSnapshot>, rusqlite::Error> {
    let mut stmt = conn.prepare(
        "SELECT json_sanitized FROM snapshots ORDER BY id DESC LIMIT ?1"
    )?;
    let rows = stmt.query_map(params![limit as i64], |row| {
        let json_str: String = row.get(0)?;
        Ok(json_str)
    })?;

    let mut snapshots = Vec::new();
    for row in rows {
        if let Ok(json_str) = row {
            if let Ok(snap) = serde_json::from_str::<QuotaSnapshot>(&json_str) {
                snapshots.push(snap);
            }
        }
    }
    Ok(snapshots)
}

pub fn clear_history(conn: &Connection) -> Result<(), rusqlite::Error> {
    conn.execute("DELETE FROM snapshots", [])?;
    Ok(())
}

pub fn load_settings(conn: &Connection) -> Result<PublicSettings, rusqlite::Error> {
    let mut stmt = conn.prepare("SELECT value FROM settings WHERE key = 'public_settings'")?;
    let mut rows = stmt.query([])?;
    if let Some(row) = rows.next()? {
        let json_str: String = row.get(0)?;
        if let Ok(settings) = serde_json::from_str::<PublicSettings>(&json_str) {
            return Ok(settings);
        }
    }
    Ok(PublicSettings::default())
}

pub fn save_settings(conn: &Connection, settings: &PublicSettings) -> Result<(), rusqlite::Error> {
    let json_str = serde_json::to_string(settings).unwrap_or_default();
    conn.execute(
        "INSERT OR REPLACE INTO settings (key, value) VALUES ('public_settings', ?1)",
        params![json_str],
    )?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::contracts::{Availability, OriginKind, ProviderId};

    #[test]
    fn test_storage_lifecycle_in_memory() {
        let conn = Connection::open_in_memory().unwrap();
        init_db(&conn).unwrap();

        // 1. Save snapshot
        let snap = QuotaSnapshot {
            schema_version: 1,
            provider_id: ProviderId::OpenaiCodex,
            origin: OriginKind::CodexAppServer,
            account_key: "acc_123".to_string(),
            account_label: "c***@openai.com".to_string(),
            status: Availability::Ok,
            observed_at: Some("2026-10-11T12:00:00Z".to_string()),
            attempted_at: Some("2026-10-11T12:00:00Z".to_string()),
            expires_at: None,
            source_version: None,
            note: None,
            windows: vec![],
            models: vec![],
            warnings: vec![],
            raw_error_code: None,
        };

        save_snapshot(&conn, &snap).unwrap();

        let history = get_history(&conn, 10).unwrap();
        assert_eq!(history.len(), 1);
        assert_eq!(history[0].account_key, "acc_123");
        assert_eq!(history[0].status, Availability::Ok);

        // 2. Settings save and load
        let mut settings = PublicSettings::default();
        settings.theme = "light".to_string();
        settings.start_with_windows = true;

        save_settings(&conn, &settings).unwrap();
        let loaded = load_settings(&conn).unwrap();
        assert_eq!(loaded.theme, "light");
        assert!(loaded.start_with_windows);

        // 3. Clear history
        clear_history(&conn).unwrap();
        let empty_history = get_history(&conn, 10).unwrap();
        assert_eq!(empty_history.len(), 0);
    }
}

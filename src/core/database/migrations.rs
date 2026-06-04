/// Database migration system with versioning
/// Migrations are applied sequentially and tracked by version number

use rusqlite::{Connection, Result};

/// Current schema version
pub const SCHEMA_VERSION: u32 = 2;

/// Initialize migrations tracking table and run pending migrations
pub fn run_migrations(conn: &mut Connection) -> Result<()> {
    // Create version table if it doesn't exist
    conn.execute(
        r#"
        CREATE TABLE IF NOT EXISTS schema_version (
            version INTEGER PRIMARY KEY,
            applied_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
            name TEXT NOT NULL
        )
        "#,
        [],
    )?;

    // Get current version
    let current_version: u32 = conn
        .query_row(
            "SELECT COALESCE(MAX(version), 0) FROM schema_version",
            [],
            |row| row.get(0),
        )
        .unwrap_or(0);

    // Run migrations in sequence
    if current_version < 1 {
        apply_v1_initial_schema(conn)?;
        conn.execute(
            "INSERT INTO schema_version (version, name) VALUES (1, 'initial_schema')",
            [],
        )?;
    }

    if current_version < 2 {
        apply_v2_add_window_color(conn)?;
        conn.execute(
            "INSERT INTO schema_version (version, name) VALUES (2, 'add_window_color')",
            [],
        )?;
    }

    Ok(())
}

/// V1: Initial schema
fn apply_v1_initial_schema(conn: &Connection) -> Result<()> {
    conn.execute_batch(
        r#"
        CREATE TABLE IF NOT EXISTS window_activity (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            hwnd INTEGER NOT NULL,
            title TEXT NOT NULL,
            class_name TEXT,
            icon_base64 TEXT,
            process_name TEXT,
            process_path TEXT,
            pid INTEGER,
            browser_name TEXT,
            browser_url TEXT,
            tags TEXT,
            left INTEGER,
            top INTEGER,
            right INTEGER,
            bottom INTEGER,
            width INTEGER,
            height INTEGER,
            is_minimized INTEGER,
            is_maximized INTEGER,
            is_visible INTEGER,
            is_focused INTEGER,
            monitor_id INTEGER,
            duration INTEGER,
            timestamp INTEGER NOT NULL
        );

        CREATE TABLE IF NOT EXISTS events (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            window_activity_id INTEGER,
            event_type INTEGER NOT NULL,
            timestamp INTEGER NOT NULL,
            duration INTEGER NOT NULL,
            FOREIGN KEY(window_activity_id) REFERENCES window_activity(id)
        );

        CREATE TABLE IF NOT EXISTS jobs (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            name TEXT NOT NULL,
            description TEXT,
            def_start_ts INTEGER,
            def_end_ts INTEGER,
            start_ts INTEGER NOT NULL,
            end_ts INTEGER NOT NULL,
            proccess_path TEXT,
            tags TEXT,
            cron TEXT,
            color TEXT NOT NULL,
            created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
        );

        CREATE TABLE IF NOT EXISTS goals (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            name TEXT NOT NULL DEFAULT '',
            description TEXT,
            ordering INTEGER NOT NULL,
            timestamp INTEGER NOT NULL,
            start_period_ts INTEGER NOT NULL,
            end_period_ts INTEGER NOT NULL,
            process TEXT NOT NULL,
            tags TEXT,
            completed BOOLEAN NOT NULL
        );

        CREATE TABLE IF NOT EXISTS tag (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            name TEXT NOT NULL,
            description TEXT,
            color TEXT NOT NULL
        );

        CREATE TABLE IF NOT EXISTS tag_to_window (
            tag_id INTEGER NOT NULL,
            process_name TEXT NOT NULL,
            FOREIGN KEY(tag_id) REFERENCES tag(id)
        );

        CREATE INDEX IF NOT EXISTS idx_window_time
            ON window_activity(timestamp);

        CREATE INDEX IF NOT EXISTS idx_events_time
            ON events(timestamp);

        CREATE INDEX IF NOT EXISTS idx_events_type
            ON events(event_type);
        "#,
    )?;

    Ok(())
}

/// V2: Add color field to window_activity table
/// This migration adds support for custom colors on tracked windows
fn apply_v2_add_window_color(conn: &Connection) -> Result<()> {
    // Check if column already exists
    let mut stmt = conn.prepare("PRAGMA table_info(window_activity)")?;
    let has_color = stmt
        .query_map([], |row| row.get::<_, String>(1))?
        .any(|col| col.ok().map(|c| c == "color").unwrap_or(false));

    if !has_color {
        conn.execute(
            r#"
            ALTER TABLE window_activity ADD COLUMN color TEXT DEFAULT 'bg-secondary/20'
            "#,
            [],
        )?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_schema_version() {
        assert_eq!(SCHEMA_VERSION, 2);
    }
}
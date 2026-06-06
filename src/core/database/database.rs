/// Database connection management and high-level operations
use std::time::Duration;
use rusqlite::{Connection, Result};
use std::sync::{Arc, Mutex};

use crate::{
    DB,
    core::{EventModel, GoalModel, JobModel, TagModel, WindowModel},
    lib::extract_icon_events,
};
use super::repositories::{WindowRepository, TagRepository, JobRepository, GoalRepository, EventRepository};

pub type Db = Arc<Mutex<Database>>;

#[derive(Debug)]
pub struct Database {
    conn: Connection,
}

pub fn with_database<F, R>(f: F) -> R
where
    F: FnOnce(&Database) -> R,
{
    let guard = DB.lock().expect("DB");
    f(&*guard)
}

pub fn with_database_mut<F, R>(f: F) -> R
where
    F: FnOnce(&mut Database) -> R,
{
    let mut guard = DB.lock().expect("DB");
    f(&mut *guard)
}

impl Database {
    /// Create and initialize database with migrations
    pub fn new(path: &str) -> Self {
        let conn = Connection::open(path).expect("Failed to open DB");

        conn.busy_timeout(Duration::from_secs(5))
            .expect("Failed to configure DB busy timeout");

        conn.execute_batch(
            r#"
            PRAGMA journal_mode = WAL;
            PRAGMA synchronous = NORMAL;
            PRAGMA temp_store = MEMORY;
            PRAGMA foreign_keys = ON;
            "#,
        )
        .expect("Failed to configure DB pragmas");

        let mut db = Self { conn };

        // Run migrations
        super::migrations::run_migrations(&mut db.conn)
            .expect("Failed to run migrations");

        db
    }

    /// Get connection reference (internal use)
    pub fn conn(&self) -> &Connection {
        &self.conn
    }

    // === WINDOW OPERATIONS ===

    /// Get all unique windows
    pub fn get_windows(&self) -> Result<Vec<(WindowModel, Vec<TagModel>)>> {
        WindowRepository::get_all_windows(&self.conn)
    }

    /// Get windows by process name
    pub fn get_windows_by_process(&self, process_name: String) -> Result<Vec<(WindowModel, Vec<TagModel>)>> {
        WindowRepository::get_by_process(&self.conn, &process_name)
    }

    /// Insert window activity record
    pub fn insert_window(&self, window: &WindowModel) -> Result<i64> {
        WindowRepository::insert(&self.conn, window)
    }

    /// Delete windows by process
    pub fn delete_window(&mut self, process_name: String) -> Result<()> {
        // Delete associated events first
        let windows = self.get_windows_by_process(process_name.clone())?;
        for (window, _) in &windows {
            if let Some(id) = window.id {
                EventRepository::delete_for_window(&self.conn, id)?;
            }
        }
        WindowRepository::delete_by_process(&self.conn, &process_name)
    }

    /// Update window color
    pub fn update_window_color(&self, window_id: i64, color: &str) -> Result<()> {
        WindowRepository::update_color(&self.conn, window_id, color)
    }

    /// Get window color
    pub fn get_window_color(&self, window_id: i64) -> Result<Option<String>> {
        WindowRepository::get_color(&self.conn, window_id)
    }

    // === TAG OPERATIONS ===

    /// Get all tags
    pub fn get_tags(&self) -> Result<Vec<TagModel>> {
        TagRepository::get_all(&self.conn)
    }

    /// Get tag by name
    pub fn get_tag_by_name(&self, name: &str) -> Result<Option<TagModel>> {
        TagRepository::get_by_name(&self.conn, name)
    }

    /// Create or get existing tag
    pub fn ensure_tag(&self, name: &str, color: &str, filter: Option<String>) -> Result<TagModel> {
        TagRepository::ensure(&self.conn, name, color, filter)
    }

    /// Get tags for a window
    pub fn get_window_tag(&self, process_name: String) -> Result<Vec<TagModel>> {
        TagRepository::get_for_process(&self.conn, &process_name)
    }

    /// Add tag to window
    pub fn add_tag_to_window(&self, tag_id: i64, process_name: String) -> Result<i64> {
        TagRepository::add_to_window(&self.conn, tag_id, &process_name)?;
        Ok(tag_id)
    }

    /// Check if tag is applied to window
    pub fn has_tag_for_window(&self, process_name: &str, tag_id: i64) -> Result<bool> {
        TagRepository::has_for_window(&self.conn, process_name, tag_id)
    }

    /// Add tag to window if not already present
    pub fn add_tag_to_window_if_missing(&self, tag_name: &str, process_name: String) -> Result<()> {
        TagRepository::add_to_window_if_missing(&self.conn, tag_name, &process_name)
    }

    /// Merge multiple tags
    pub fn merge_tags(&mut self, tags: &[TagModel]) -> Result<usize> {
        TagRepository::merge_many(&self.conn, tags)
    }

    pub fn update_tag(&self, tag: &TagModel) -> Result<usize> {
        TagRepository::update(&self.conn, tag)
    }

    pub fn delete_tag(&self, id: i64) -> Result<()> {
        TagRepository::delete(&self.conn, id)
    }

    // === JOB OPERATIONS ===

    /// Get all jobs
    pub fn get_jobs(&self) -> Result<Vec<JobModel>> {
        JobRepository::get_all(&self.conn)
    }

    /// Insert job
    pub fn insert_jobs(&mut self, job: &JobModel) -> Result<()> {
        JobRepository::insert(&self.conn, job)?;
        Ok(())
    }

    /// Save job (insert)
    pub fn save_job(&mut self, job: &JobModel) -> Result<i64> {
        JobRepository::insert(&self.conn, job)
    }

    /// Update job
    pub fn update_job(&self, job: &JobModel) -> Result<()> {
        JobRepository::update(&self.conn, job)?;
        Ok(())
    }

    /// Delete job
    pub fn delete_job(&self, id: i64) -> Result<()> {
        JobRepository::delete(&self.conn, id)
    }

    /// Get jobs for day (stub)
    pub fn get_jobs_for_day(&self, _day_start_ts: i64, _day_end_ts: i64) -> Result<Vec<JobModel>> {
        JobRepository::get_all(&self.conn)
    }

    // === GOAL OPERATIONS ===

    /// Get all goals
    pub fn get_goals(&self) -> Result<Vec<GoalModel>> {
        GoalRepository::get_all(&self.conn)
    }

    /// Insert goal
    pub fn insert_goal(&mut self, goal: &GoalModel) -> Result<i64> {
        GoalRepository::insert(&self.conn, goal)
    }

    /// Update goal
    pub fn update_goal(&self, goal: &GoalModel) -> Result<()> {
        GoalRepository::update(&self.conn, goal)
    }

    /// Delete goal
    pub fn delete_goal(&self, id: i64) -> Result<()> {
        GoalRepository::delete(&self.conn, id)
    }

    // === EVENT OPERATIONS ===

    /// Insert single event
    pub fn insert_event(&self, event: &EventModel) -> Result<()> {
        EventRepository::insert(&self.conn, event)
    }

    /// Insert multiple events
    pub fn insert_events(&mut self, events: &[EventModel]) -> Result<()> {
        EventRepository::insert_batch(&mut self.conn, events)
    }

    /// Get all events
    pub fn get_all_events(&self) -> Result<Vec<EventModel>> {
        EventRepository::get_all(&self.conn)
    }

    /// Get events in a time range
    pub fn get_events_in_range(&self, from_ts: i64, to_ts: i64) -> Result<Vec<EventModel>> {
        EventRepository::get_in_range(&self.conn, from_ts, to_ts)
    }

    /// Get events since a specific timestamp
    pub fn get_events_since(&self, timestamp: i64, limit: i64) -> Result<Vec<EventModel>> {
        EventRepository::get_since(&self.conn, timestamp, limit)
    }

    // === ANALYTICS ===

    /// Get top processes by count
    pub fn get_top_processes(&self) -> Result<Vec<(String, i64)>> {
        let mut stmt = self.conn.prepare(
            "SELECT process_name, COUNT(*) FROM window_activity GROUP BY process_name ORDER BY COUNT(*) DESC"
        )?;
        let rows = stmt.query_map([], |row| Ok((row.get(0)?, row.get(1)?)))?;
        Ok(rows.filter_map(Result::ok).collect())
    }

    /// Clean up old records
    pub fn cleanup_old(&self, older_than_ts: i64) -> Result<()> {
        self.conn.execute(
            "DELETE FROM window_activity WHERE timestamp < ?1",
            [older_than_ts],
        )?;
        Ok(())
    }

    /// Get total time by process
    pub fn get_total_time_by_process(&self) -> Result<Vec<(String, i64)>> {
        let mut stmt = self.conn.prepare(
            r#"
            SELECT w.process_name, SUM(e.duration)
            FROM events e
            JOIN window_activity w ON e.window_activity_id = w.id
            GROUP BY w.process_name
            ORDER BY SUM(e.duration) DESC
            "#
        )?;
        let rows = stmt.query_map([], |row| Ok((row.get(0)?, row.get(1)?)))?;
        Ok(rows.filter_map(Result::ok).collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_database_creation() {
        // Database creation is tested in integration tests
        // This is a placeholder
    }
}

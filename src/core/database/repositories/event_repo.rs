/// Event Repository
/// Manages event records

use rusqlite::{Connection, Result};
use crate::core::EventModel;
use super::super::schema::events as schema;

pub struct EventRepository;

impl EventRepository {
    /// Insert a single event
    pub fn insert(conn: &Connection, event: &EventModel) -> Result<()> {
        let window_id: Option<i64> = None;
        
        conn.execute(
            &format!(
                r#"
                INSERT INTO {} ({}, {}, {}, {})
                VALUES (?1, ?2, ?3, ?4)
                "#,
                schema::TABLE,
                schema::COL_WINDOW_ACTIVITY_ID,
                schema::COL_EVENT_TYPE,
                schema::COL_TIMESTAMP,
                schema::COL_DURATION
            ),
            (
                window_id,
                event.event_type.as_i32(),
                event.timestamp as i64,
                event.duration as i64,
            ),
        )?;

        Ok(())
    }

    /// Insert multiple events in a batch with transaction protection
    pub fn insert_batch(conn: &mut Connection, events: &[EventModel]) -> Result<()> {
        if events.is_empty() {
            return Ok(());
        }

        let tx = conn.transaction()?;
        
        for event in events {
            let window_id: Option<i64> = None;
            
            tx.execute(
                &format!(
                    r#"
                    INSERT INTO {} ({}, {}, {}, {})
                    VALUES (?1, ?2, ?3, ?4)
                    "#,
                    schema::TABLE,
                    schema::COL_WINDOW_ACTIVITY_ID,
                    schema::COL_EVENT_TYPE,
                    schema::COL_TIMESTAMP,
                    schema::COL_DURATION
                ),
                (
                    window_id,
                    event.event_type.as_i32(),
                    event.timestamp as i64,
                    event.duration as i64,
                ),
            )?;
        }

        tx.commit()?;
        Ok(())
    }

    /// Delete events for a window
    pub fn delete_for_window(conn: &Connection, window_id: i64) -> Result<()> {
        conn.execute(
            &format!(
                "DELETE FROM {} WHERE {} = ?1",
                schema::TABLE,
                schema::COL_WINDOW_ACTIVITY_ID
            ),
            [window_id],
        )?;
        Ok(())
    }

    /// Get all events
    pub fn get_all(conn: &Connection) -> Result<Vec<EventModel>> {
        let mut stmt = conn.prepare(
            &format!(
                "SELECT {} FROM {} ORDER BY {} DESC",
                &[
                    schema::COL_ID,
                    schema::COL_WINDOW_ACTIVITY_ID,
                    schema::COL_EVENT_TYPE,
                    schema::COL_TIMESTAMP,
                    schema::COL_DURATION,
                ].join(", "),
                schema::TABLE,
                schema::COL_TIMESTAMP
            )
        )?;

        let rows = stmt.query_map([], |row| {
            let event_type_i32: i32 = row.get(2)?;
            let event_type = match event_type_i32 {
                0 => crate::core::EventType::Idle,
                1 => crate::core::EventType::WindowSwitch,
                2 => crate::core::EventType::Keyboard,
                3 => crate::core::EventType::Mouse,
                _ => crate::core::EventType::Idle,
            };

            Ok(EventModel {
                window: None,
                event_type,
                timestamp: row.get::<_, i64>(3)? as u64,
                duration: row.get::<_, i64>(4)? as u64,
            })
        })?;

        Ok(rows.filter_map(Result::ok).collect())
    }

    /// Get events within a time range
    pub fn get_in_range(conn: &Connection, from_ts: i64, to_ts: i64) -> Result<Vec<EventModel>> {
        let mut stmt = conn.prepare(
            &format!(
                "SELECT {} FROM {} WHERE {} >= ?1 AND {} <= ?2 ORDER BY {} DESC",
                &[
                    schema::COL_ID,
                    schema::COL_WINDOW_ACTIVITY_ID,
                    schema::COL_EVENT_TYPE,
                    schema::COL_TIMESTAMP,
                    schema::COL_DURATION,
                ].join(", "),
                schema::TABLE,
                schema::COL_TIMESTAMP,
                schema::COL_TIMESTAMP,
                schema::COL_TIMESTAMP
            )
        )?;

        let rows = stmt.query_map([from_ts, to_ts], |row| {
            let event_type_i32: i32 = row.get(2)?;
            let event_type = match event_type_i32 {
                0 => crate::core::EventType::Idle,
                1 => crate::core::EventType::WindowSwitch,
                2 => crate::core::EventType::Keyboard,
                3 => crate::core::EventType::Mouse,
                _ => crate::core::EventType::Idle,
            };

            Ok(EventModel {
                window: None,
                event_type,
                timestamp: row.get::<_, i64>(3)? as u64,
                duration: row.get::<_, i64>(4)? as u64,
            })
        })?;

        Ok(rows.filter_map(Result::ok).collect())
    }

    /// Get events since a specific timestamp
    pub fn get_since(conn: &Connection, timestamp: i64, limit: i64) -> Result<Vec<EventModel>> {
        let mut stmt = conn.prepare(
            &format!(
                "SELECT {} FROM {} WHERE {} >= ?1 ORDER BY {} DESC LIMIT ?2",
                &[
                    schema::COL_ID,
                    schema::COL_WINDOW_ACTIVITY_ID,
                    schema::COL_EVENT_TYPE,
                    schema::COL_TIMESTAMP,
                    schema::COL_DURATION,
                ].join(", "),
                schema::TABLE,
                schema::COL_TIMESTAMP,
                schema::COL_TIMESTAMP
            )
        )?;

        let rows = stmt.query_map([timestamp, limit], |row| {
            let event_type_i32: i32 = row.get(2)?;
            let event_type = match event_type_i32 {
                0 => crate::core::EventType::Idle,
                1 => crate::core::EventType::WindowSwitch,
                2 => crate::core::EventType::Keyboard,
                3 => crate::core::EventType::Mouse,
                _ => crate::core::EventType::Idle,
            };

            Ok(EventModel {
                window: None,
                event_type,
                timestamp: row.get::<_, i64>(3)? as u64,
                duration: row.get::<_, i64>(4)? as u64,
            })
        })?;

        Ok(rows.filter_map(Result::ok).collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_event_schema() {
        assert_eq!(schema::TABLE, "events");
    }
}

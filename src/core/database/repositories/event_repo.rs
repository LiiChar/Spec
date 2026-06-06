/// Event Repository
/// Manages event records

use rusqlite::{Connection, Result};
use crate::{core::{EventModel, EventType, Rect, TagModel, TagRepository, WindowBrowser, WindowDesktop, WindowModel, WindowRepository, WindowVariant, tag_repo::attach_tags}, lib::extract_icon_events};
use super::super::schema::events as schema;

pub struct EventRepository;

impl EventRepository {
    /// Insert a single event
    pub fn insert(conn: &Connection, event: &EventModel) -> Result<()> {
        let mut window_id: Option<i64> = None;

        if event.window.is_some() {
            window_id = Some(WindowRepository::insert(&conn, event.window.as_ref().unwrap())?);
        }
        
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
            let mut window_id: Option<i64> = None;

            if event.window.is_some() {
                window_id = Some(WindowRepository::insert(&tx, event.window.as_ref().unwrap())?);
            }
            
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
            r#"
            SELECT
                w.hwnd, w.title, w.class_name,
                w.process_name, w.process_path, w.pid,
                w.left, w.top, w.right, w.bottom, w.width, w.height,
                w.is_minimized, w.is_maximized, w.is_visible, w.is_focused,
                w.monitor_id, w.timestamp, w.duration,
                w.browser_name, w.browser_url,
                e.event_type AS event_type,
                e.timestamp AS event_timestamp,
                e.duration AS event_duration,
                w.icon_base64,
                w.color
            FROM events e
            LEFT JOIN window_activity w ON e.window_activity_id = w.id
            ORDER BY e.timestamp ASC
            "#,
        )?;

        let events: Vec<EventModel> = stmt
            .query_map([], row_to_event)?
            .filter_map(Result::ok)
            .map(|mut event| {
                if let Some(window) = &mut event.window {
                    window.tags = TagRepository::get_for_process(
                        conn,
                        &window.process_name,
                    )
                    .unwrap_or_default();
                }

                event
            })
            .collect();

        let events = extract_icon_events(events);

        Ok(events)
    }

    /// Get events within a time range
    pub fn get_in_range(
        conn: &Connection,
        from_ts: i64,
        to_ts: i64,
    ) -> Result<Vec<EventModel>> {
        let mut stmt = conn.prepare(
            r#"
            SELECT
                w.hwnd, w.title, w.class_name,
                w.process_name, w.process_path, w.pid,
                w.left, w.top, w.right, w.bottom, w.width, w.height,
                w.is_minimized, w.is_maximized, w.is_visible, w.is_focused,
                w.monitor_id, w.timestamp, w.duration,
                w.browser_name, w.browser_url,
                e.event_type AS event_type,
                e.timestamp AS event_timestamp,
                e.duration AS event_duration,
                w.icon_base64,
                w.color
            FROM events e
            LEFT JOIN window_activity w ON e.window_activity_id = w.id
            WHERE e.timestamp <= ?2
            AND (e.timestamp + e.duration) >= ?1
            ORDER BY e.timestamp ASC
            "#,
        )?;

        let events: Vec<EventModel> = stmt
            .query_map([from_ts, to_ts], row_to_event)?
            .filter_map(Result::ok)
            .collect();

        let events = attach_tags(conn, events);
        let events = extract_icon_events(events);

        Ok(events)
    }

    /// Get events since a specific timestamp
    pub fn get_since(
        conn: &Connection,
        timestamp: i64,
        limit: i64,
    ) -> Result<Vec<EventModel>> {
        let mut stmt = conn.prepare(
            r#"
            SELECT
                w.hwnd, w.title, w.class_name,
                w.process_name, w.process_path, w.pid,
                w.left, w.top, w.right, w.bottom, w.width, w.height,
                w.is_minimized, w.is_maximized, w.is_visible, w.is_focused,
                w.monitor_id, w.timestamp, w.duration,
                w.browser_name, w.browser_url,
                e.event_type AS event_type,
                e.timestamp AS event_timestamp,
                e.duration AS event_duration,
                w.icon_base64,
                w.color
            FROM events e
            LEFT JOIN window_activity w ON e.window_activity_id = w.id
            WHERE e.timestamp >= ?1
            ORDER BY e.timestamp DESC
            LIMIT ?2
            "#,
        )?;

        let mut events: Vec<EventModel> = stmt
            .query_map([timestamp, limit], row_to_event)?
            .filter_map(Result::ok)
            .collect();

        events = attach_tags(conn, events);
        events = extract_icon_events(events);

        events.reverse();

        Ok(events)
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

fn row_to_event(row: &rusqlite::Row) -> Result<EventModel> {
    let hwnd: Option<i64> = row.get("hwnd").ok();

    let window = if hwnd.is_some() {
        let browser_name: Option<String> = row.get("browser_name")?;
        let browser_url: Option<String> = row.get("browser_url")?;
        let variant = if let (Some(browser_name), Some(browser_url)) =
            (browser_name.clone(), browser_url.clone())
        {
            WindowVariant::Browser(WindowBrowser {
                browser: browser_name,
                url: browser_url,
            })
        } else {
            WindowVariant::Desktop(WindowDesktop {})
        };

        Some(WindowModel {
            id: None,
            hwnd: row.get("hwnd")?,
            title: row.get("title")?,
            class_name: row.get("class_name")?,
            process_name: row.get("process_name")?,
            process_path: row.get("process_path")?,
            pid: row.get("pid")?,
            rect: Rect {
                left: row.get("left")?,
                top: row.get("top")?,
                right: row.get("right")?,
                bottom: row.get("bottom")?,
                width: row.get("width")?,
                height: row.get("height")?,
            },
            variant,
            is_minimized: row.get("is_minimized")?,
            is_maximized: row.get("is_maximized")?,
            is_visible: row.get("is_visible")?,
            is_focused: row.get("is_focused")?,
            monitor_id: row.get("monitor_id")?,
            timestamp: row.get::<_, i64>("timestamp")? as u64,
            duration: row.get::<_, i64>("duration")? as u64,
            icon_base64: row.get("icon_base64").ok(),
            color: row.get("color").unwrap_or_else(|_| "rgba(0,0,0,1)".to_string()),
            tags: Vec::new(),
        })
    } else {
        None
    };

    Ok(EventModel {
        window,
        event_type: event_type_from_i32(row.get("event_type")?),
        timestamp: row.get::<_, i64>("event_timestamp")? as u64,
        duration: row.get::<_, i64>("event_duration")? as u64,
    })
}


fn event_type_from_i32(value: i32) -> EventType {
    match value {
        0 => EventType::Idle,
        1 => EventType::WindowSwitch,
        2 => EventType::Keyboard,
        3 => EventType::Mouse,
        _ => EventType::Idle,
    }
}
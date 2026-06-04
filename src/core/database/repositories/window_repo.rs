/// Window Activity Repository
/// Provides type-safe access to window tracking data

use rusqlite::{Connection, Result};
use crate::core::{WindowModel, WindowVariant, WindowBrowser, WindowDesktop, Rect, TagModel};
use crate::lib::extract_icon;
use crate::core::window::icon_file_name;
use super::super::schema::window_activity as schema;

pub struct WindowRepository;

impl WindowRepository {
    /// Get all unique windows (latest entry per process)
    pub fn get_all_windows(conn: &Connection) -> Result<Vec<(WindowModel, Vec<TagModel>)>> {
        let mut stmt = conn.prepare(
            r#"
            WITH ranked AS (
                SELECT
                    id, hwnd, title, class_name, icon_base64,
                    process_name, process_path, pid,
                    left, top, right, bottom, width, height,
                    is_minimized, is_maximized, is_visible, is_focused,
                    monitor_id, duration, timestamp, browser_name, browser_url, tags, color,
                    ROW_NUMBER() OVER (
                        PARTITION BY LOWER(TRIM(process_name))
                        ORDER BY timestamp DESC, id DESC
                    ) AS rn
                FROM window_activity
            )
            SELECT
                hwnd, title, class_name, icon_base64,
                process_name, process_path, pid,
                browser_name, browser_url,
                left, top, right, bottom, width, height,
                is_minimized, is_maximized, is_visible, is_focused,
                monitor_id, duration, timestamp, id, tags, color
            FROM ranked
            WHERE rn = 1
            ORDER BY timestamp DESC
            "#,
        )?;

        let windows = stmt.query_map([], |row| {
            Self::parse_window_row(conn, row)
        })?;

        Ok(windows.collect::<Result<Vec<_>>>()?)
    }

    /// Get windows by process name
    pub fn get_by_process(conn: &Connection, process_name: &str) -> Result<Vec<(WindowModel, Vec<TagModel>)>> {
        let mut stmt = conn.prepare(
            &format!(
                r#"
                SELECT
                    hwnd, title, class_name, icon_base64,
                    process_name, process_path, pid,
                    browser_name, browser_url,
                    left, top, right, bottom, width, height,
                    is_minimized, is_maximized, is_visible, is_focused,
                    monitor_id, duration, timestamp, id, tags, color
                FROM {}
                WHERE LOWER(TRIM(process_name)) = LOWER(TRIM(?1))
                ORDER BY timestamp DESC, id DESC
                "#,
                schema::TABLE
            )
        )?;

        let windows = stmt.query_map([process_name], |row| {
            Self::parse_window_row(conn, row)
        })?;

        Ok(windows.collect::<Result<Vec<_>>>()?)
    }

    /// Insert a new window activity record
    pub fn insert(conn: &Connection, window: &WindowModel) -> Result<i64> {
        let icon_ref = icon_file_name(&window.process_path);

        conn.execute(
            &format!(
                r#"
                INSERT INTO {} (
                    hwnd, title, class_name, icon_base64,
                    process_name, process_path, pid,
                    browser_name, browser_url,
                    left, top, right, bottom, width, height,
                    is_minimized, is_maximized, is_visible, is_focused,
                    monitor_id, timestamp, duration, color
                )
                VALUES (
                    :hwnd, :title, :class_name, :icon_base64,
                    :process_name, :process_path, :pid,
                    :browser_name, :browser_url,
                    :left, :top, :right, :bottom, :width, :height,
                    :is_minimized, :is_maximized, :is_visible, :is_focused,
                    :monitor_id, :timestamp, :duration, :color
                )
                "#,
                schema::TABLE
            ),
            rusqlite::named_params! {
                ":hwnd": window.hwnd,
                ":title": &window.title,
                ":class_name": &window.class_name,
                ":icon_base64": icon_ref,
                ":process_name": &window.process_name,
                ":process_path": &window.process_path,
                ":pid": window.pid,
                ":browser_name": match &window.variant {
                    WindowVariant::Browser(b) => Some(b.browser.clone()),
                    _ => None,
                },
                ":browser_url": match &window.variant {
                    WindowVariant::Browser(b) => Some(b.url.clone()),
                    _ => None,
                },
                ":left": window.rect.left,
                ":top": window.rect.top,
                ":right": window.rect.right,
                ":bottom": window.rect.bottom,
                ":width": window.rect.width,
                ":height": window.rect.height,
                ":is_minimized": window.is_minimized as i32,
                ":is_maximized": window.is_maximized as i32,
                ":is_visible": window.is_visible as i32,
                ":is_focused": window.is_focused as i32,
                ":monitor_id": window.monitor_id.map(|v| v as i32),
                ":timestamp": window.timestamp as i64,
                ":duration": window.duration as i64,
                ":color": "bg-secondary/20",
            },
        )?;

        Ok(conn.last_insert_rowid())
    }

    /// Delete all windows for a process
    pub fn delete_by_process(conn: &Connection, process_name: &str) -> Result<()> {
        conn.execute(
            &format!(
                "DELETE FROM {} WHERE LOWER(TRIM(process_name)) = LOWER(TRIM(?1))",
                schema::TABLE
            ),
            [process_name],
        )?;
        Ok(())
    }

    /// Update window color
    pub fn update_color(conn: &Connection, window_id: i64, color: &str) -> Result<()> {
        conn.execute(
            &format!(
                "UPDATE {} SET {} = ?1 WHERE {} = ?2",
                schema::TABLE, schema::COL_COLOR, schema::COL_ID
            ),
            (color, window_id),
        )?;
        Ok(())
    }

    /// Get color for a window
    pub fn get_color(conn: &Connection, window_id: i64) -> Result<Option<String>> {
        let mut stmt = conn.prepare(
            &format!(
                "SELECT {} FROM {} WHERE {} = ?1",
                schema::COL_COLOR, schema::TABLE, schema::COL_ID
            )
        )?;

        match stmt.query_row([window_id], |row| row.get::<_, String>(0)) {
            Ok(color) => Ok(Some(color)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e),
        }
    }

    // Helper to parse window from row
    fn parse_window_row(
        conn: &Connection,
        row: &rusqlite::Row,
    ) -> Result<(WindowModel, Vec<TagModel>)> {
        let hwnd: i64 = row.get(0)?;
        let title: String = row.get(1)?;
        let class_name: String = row.get(2)?;
        let icon_base64: Option<String> = row.get(3)?;
        let process_name: String = row.get(4)?;
        let process_path: String = row.get(5)?;
        let pid: i32 = row.get(6)?;
        let browser_name: Option<String> = row.get(7)?;
        let browser_url: Option<String> = row.get(8)?;
        let left: i32 = row.get(9)?;
        let top: i32 = row.get(10)?;
        let right: i32 = row.get(11)?;
        let bottom: i32 = row.get(12)?;
        let width: i32 = row.get(13)?;
        let height: i32 = row.get(14)?;
        let is_minimized: i32 = row.get(15)?;
        let is_maximized: i32 = row.get(16)?;
        let is_visible: i32 = row.get(17)?;
        let is_focused: i32 = row.get(18)?;
        let monitor_id: Option<i32> = row.get(19)?;
        let duration: i64 = row.get(20)?;
        let timestamp: i64 = row.get(21)?;
        let id: i64 = row.get(22)?;
        let tags_json: Option<String> = row.get(23)?;
        let color: String = row.get(24)?;

        let rect = Rect {
            left,
            top,
            right,
            bottom,
            width,
            height,
        };

        let tags = parse_tags_json(tags_json);
        let icon_base64 = extract_icon(icon_base64.unwrap_or_default());

        Ok((
            WindowModel {
                id: Some(id),
                hwnd: hwnd.try_into().unwrap(),
                title,
                class_name,
                icon_base64,
                process_name,
                process_path,
                pid: pid.try_into().unwrap(),
                rect,
                variant: if let (Some(browser_name), Some(browser_url)) = (browser_name, browser_url) {
                    WindowVariant::Browser(WindowBrowser {
                        browser: browser_name,
                        url: browser_url,
                    })
                } else {
                    WindowVariant::Desktop(WindowDesktop {})
                },
                is_minimized: is_minimized != 0,
                is_maximized: is_maximized != 0,
                is_visible: is_visible != 0,
                is_focused: is_focused != 0,
                monitor_id: monitor_id.map(|v| v as u32),
                timestamp: timestamp as u64,
                duration: duration as u64,
                color,
            },
            tags,
        ))
    }
}

fn parse_tags_json(raw: Option<String>) -> Vec<TagModel> {
    raw.and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_tags_json() {
        let tags = parse_tags_json(None);
        assert_eq!(tags.len(), 0);
    }
}

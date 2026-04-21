use std::time::Duration;

use rusqlite::{Connection, Result};
use std::sync::{Arc, Mutex};

use crate::core::{EventModel, EventType, Rect, WindowModel};

pub type Db = Arc<Mutex<WindowsDatabase>>;

pub struct WindowsDatabase {
    conn: Connection,
}

impl WindowsDatabase {
    /// Создание + инициализация БД
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

        let db = Self { conn };
        db.init().expect("Failed to init DB");

        db
    }

    /// Создание таблиц
  fn init(&self) -> Result<()> {
        self.conn.execute_batch(
            r#"
            CREATE TABLE IF NOT EXISTS window_activity (
                id INTEGER PRIMARY KEY AUTOINCREMENT,

                hwnd INTEGER NOT NULL,

                title TEXT NOT NULL,
                class_name TEXT,

                process_name TEXT,
                process_path TEXT,
                pid INTEGER,

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

            -- 👇 НОВАЯ ТАБЛИЦА
            CREATE TABLE IF NOT EXISTS events (
                id INTEGER PRIMARY KEY AUTOINCREMENT,

                window_activity_id INTEGER,
                event_type INTEGER NOT NULL,

                timestamp INTEGER NOT NULL,
                duration INTEGER NOT NULL,

                FOREIGN KEY(window_activity_id) REFERENCES window_activity(id)
            );

            CREATE INDEX IF NOT EXISTS idx_window_time
            ON window_activity(timestamp);

            CREATE INDEX IF NOT EXISTS idx_events_time
            ON events(timestamp);

            CREATE INDEX IF NOT EXISTS idx_events_type
            ON events(event_type);
            "#,
        )?;

        let mut stmt = self.conn.prepare("PRAGMA table_info(events)")?;
        let mut needs_migration = false;

        let columns = stmt.query_map([], |row| {
            let name: String = row.get(1)?;
            let notnull: i32 = row.get(3)?;
            Ok((name, notnull))
        })?;

        for column in columns {
            let (name, notnull) = column?;
            if name == "window_activity_id" && notnull == 1 {
                needs_migration = true;
                break;
            }
        }

        if needs_migration {
            self.conn.execute_batch(
                r#"
                ALTER TABLE events RENAME TO events_old;

                CREATE TABLE events (
                    id INTEGER PRIMARY KEY AUTOINCREMENT,
                    window_activity_id INTEGER,
                    event_type INTEGER NOT NULL,
                    timestamp INTEGER NOT NULL,
                    duration INTEGER NOT NULL,
                    FOREIGN KEY(window_activity_id) REFERENCES window_activity(id)
                );

                INSERT INTO events (id, window_activity_id, event_type, timestamp, duration)
                SELECT id,
                       CASE WHEN window_activity_id = -1 THEN NULL ELSE window_activity_id END,
                       event_type,
                       timestamp,
                       duration
                FROM events_old;

                DROP TABLE events_old;
                "#,
            )?;
        }

        Ok(())
    }

    /// Вставка окна в БД
    pub fn insert_window(&self, w: &WindowModel) -> Result<i64> {
        self.conn.execute(
            r#"
            INSERT INTO window_activity (
                hwnd, title, class_name,
                process_name, process_path, pid,
                left, top, right, bottom, width, height,
                is_minimized, is_maximized, is_visible, is_focused,
                monitor_id, timestamp, duration
            )
            VALUES (
                :hwnd, :title, :class_name,
                :process_name, :process_path, :pid,
                :left, :top, :right, :bottom, :width, :height,
                :is_minimized, :is_maximized, :is_visible, :is_focused,
                :monitor_id, :timestamp, :duration
            )
            "#,
            rusqlite::named_params! {
                ":hwnd": w.hwnd,
                ":title": &w.title,
                ":class_name": &w.class_name,
                ":process_name": &w.process_name,
                ":process_path": &w.process_path,
                ":pid": w.pid,
                ":left": w.rect.left,
                ":top": w.rect.top,
                ":right": w.rect.right,
                ":bottom": w.rect.bottom,
                ":width": w.rect.width,
                ":height": w.rect.height,
                ":is_minimized": w.is_minimized as i32,
                ":is_maximized": w.is_maximized as i32,
                ":is_visible": w.is_visible as i32,
                ":is_focused": w.is_focused as i32,
                ":monitor_id": w.monitor_id.map(|v| v as i32),
                ":timestamp": w.timestamp as i64,
                ":duration": w.duration as i64,
            },
        )?;

        Ok(self.conn.last_insert_rowid())
    }

    pub fn insert_event(&self, event: &EventModel) -> Result<()> {
        // 1. сохраняем окно
        let window_id: Option<i64> = match &event.window {
            Some(w) => Some(self.insert_window(w)?),
            None => None,
        };

        // 2. сохраняем событие
        self.conn.execute(
            r#"
            INSERT INTO events (
                window_activity_id,
                event_type,
                timestamp,
                duration
            )
            VALUES (?1, ?2, ?3, ?4)
            "#,
            (
                window_id,
                event.event_type.as_i32(),
                event.timestamp as i64,
                event.duration as i64,
            ),
        )?;

        Ok(())
    }

    pub fn insert_events(&mut self, events: &[EventModel]) -> Result<()> {
        if events.is_empty() {
            return Ok(());
        }

        let tx = self.conn.transaction()?;

        for event in events {
            let window_id: Option<i64> = match &event.window {
                Some(w) => {
                    tx.execute(
                        r#"
                        INSERT INTO window_activity (
                            hwnd, title, class_name,
                            process_name, process_path, pid,
                            left, top, right, bottom, width, height,
                            is_minimized, is_maximized, is_visible, is_focused,
                            monitor_id, timestamp, duration
                        )
                        VALUES (
                            :hwnd, :title, :class_name,
                            :process_name, :process_path, :pid,
                            :left, :top, :right, :bottom, :width, :height,
                            :is_minimized, :is_maximized, :is_visible, :is_focused,
                            :monitor_id, :timestamp, :duration
                        )
                        "#,
                        rusqlite::named_params! {
                            ":hwnd": w.hwnd,
                            ":title": &w.title,
                            ":class_name": &w.class_name,
                            ":process_name": &w.process_name,
                            ":process_path": &w.process_path,
                            ":pid": w.pid,
                            ":left": w.rect.left,
                            ":top": w.rect.top,
                            ":right": w.rect.right,
                            ":bottom": w.rect.bottom,
                            ":width": w.rect.width,
                            ":height": w.rect.height,
                            ":is_minimized": w.is_minimized as i32,
                            ":is_maximized": w.is_maximized as i32,
                            ":is_visible": w.is_visible as i32,
                            ":is_focused": w.is_focused as i32,
                            ":monitor_id": w.monitor_id.map(|v| v as i32),
                            ":timestamp": w.timestamp as i64,
                            ":duration": w.duration as i64,
                        },
                    )?;

                    Some(tx.last_insert_rowid())
                }
                None => None,
            };

            tx.execute(
                r#"
                INSERT INTO events (
                    window_activity_id,
                    event_type,
                    timestamp,
                    duration
                )
                VALUES (?1, ?2, ?3, ?4)
                "#,
                (
                    window_id,
                    event.event_type.as_i32(),
                    event.timestamp as i64,
                    event.duration as i64,
                ),
            )?;
        }

        tx.commit()
    }

    fn row_to_event(row: &rusqlite::Row) -> Result<EventModel> {
        let window = match row.get::<_, Option<i64>>(0)? {
            Some(_) => Some(WindowModel {
                hwnd: row.get(0)?,
                title: row.get(1)?,
                class_name: row.get(2)?,
                process_name: row.get(3)?,
                process_path: row.get(4)?,
                pid: row.get(5)?,
                rect: Rect {
                    left: row.get(6)?,
                    top: row.get(7)?,
                    right: row.get(8)?,
                    bottom: row.get(9)?,
                    width: row.get(10)?,
                    height: row.get(11)?,
                },
                is_minimized: row.get(12)?,
                is_maximized: row.get(13)?,
                is_visible: row.get(14)?,
                is_focused: row.get(15)?,
                monitor_id: row.get(16)?,
                timestamp: row.get::<_, i64>(17)? as u64,
                duration: row.get::<_, i64>(18)? as u64,
            } ),
            None => None,
        };

        let event_type = Self::event_type_from_i32(row.get(19)?);
        let timestamp = row.get::<_, i64>(20)? as u64;
        let duration = row.get::<_, i64>(21)? as u64;

        Ok(EventModel {
            window,
            event_type,
            timestamp,
            duration,
        })
    }

    /// Простой аналитический запрос: топ процессов
    pub fn get_top_processes(&self) -> Result<Vec<(String, i64)>> {
        let mut stmt = self.conn.prepare(
            r#"
            SELECT process_name, COUNT(*)
            FROM window_activity
            GROUP BY process_name
            ORDER BY COUNT(*) DESC
            "#,
        )?;

        let rows = stmt.query_map([], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, i64>(1)?,
            ))
        })?;

        Ok(rows.filter_map(Result::ok).collect())
    }

    /// Очистка старых данных (например, старше N дней)
    pub fn cleanup_old(&self, older_than_ts: i64) -> Result<()> {
        self.conn.execute(
            "DELETE FROM window_activity WHERE timestamp < ?1",
            [older_than_ts],
        )?;

        Ok(())
    }
    /// Получить все события с информацией об окнах
    pub fn get_all_events(&self) -> Result<Vec<EventModel>> {
        let mut stmt = self.conn.prepare(
            r#"
            SELECT 
                w.hwnd, w.title, w.class_name,
                w.process_name, w.process_path, w.pid,
                w.left, w.top, w.right, w.bottom, w.width, w.height,
                w.is_minimized, w.is_maximized, w.is_visible, w.is_focused,
                w.monitor_id, w.timestamp, w.duration,
                e.event_type, e.timestamp, e.duration
            FROM events e
            LEFT JOIN window_activity w ON e.window_activity_id = w.id
            ORDER BY e.timestamp ASC
            "#,
        )?;

        let events = stmt.query_map([], |row| Self::row_to_event(row))?;

        Ok(events.filter_map(Result::ok).collect())
    }

    /// Получить события за временной интервал
    pub fn get_events_in_range(&self, from_ts: i64, to_ts: i64) -> Result<Vec<EventModel>> {
        let mut stmt = self.conn.prepare(
            r#"
            SELECT 
                w.hwnd, w.title, w.class_name,
                w.process_name, w.process_path, w.pid,
                w.left, w.top, w.right, w.bottom, w.width, w.height,
                w.is_minimized, w.is_maximized, w.is_visible, w.is_focused,
                w.monitor_id, w.timestamp, w.duration,
                e.event_type, e.timestamp, e.duration
            FROM events e
            LEFT JOIN window_activity w ON e.window_activity_id = w.id
            WHERE e.timestamp BETWEEN ?1 AND ?2
            ORDER BY e.timestamp ASC
            "#,
        )?;

        let events = stmt.query_map([from_ts, to_ts], |row| Self::row_to_event(row))?;

        Ok(events.filter_map(Result::ok).collect())
    }

    /// Получить события по типу
    pub fn get_events_by_type(&self, event_type: EventType) -> Result<Vec<EventModel>> {
        let type_i32 = event_type.as_i32();
        let mut stmt = self.conn.prepare(
            r#"
            SELECT 
                w.hwnd, w.title, w.class_name,
                w.process_name, w.process_path, w.pid,
                w.left, w.top, w.right, w.bottom, w.width, w.height,
                w.is_minimized, w.is_maximized, w.is_visible, w.is_focused,
                w.monitor_id, w.timestamp, w.duration,
                e.event_type, e.timestamp, e.duration
            FROM events e
            LEFT JOIN window_activity w ON e.window_activity_id = w.id
            WHERE e.event_type = ?1
            ORDER BY e.timestamp ASC
            "#,
        )?;

        let events = stmt.query_map([type_i32], |row| Self::row_to_event(row))?;

        Ok(events.filter_map(Result::ok).collect())
    }

    /// Получить события по процессу
    pub fn get_events_by_process(&self, process_name: &str) -> Result<Vec<EventModel>> {
        let mut stmt = self.conn.prepare(
            r#"
            SELECT 
                w.hwnd, w.title, w.class_name,
                w.process_name, w.process_path, w.pid,
                w.left, w.top, w.right, w.bottom, w.width, w.height,
                w.is_minimized, w.is_maximized, w.is_visible, w.is_focused,
                w.monitor_id, w.timestamp, w.duration,
                e.event_type, e.timestamp, e.duration
            FROM events e
            JOIN window_activity w ON e.window_activity_id = w.id
            WHERE w.process_name = ?1
            ORDER BY e.timestamp ASC
            "#,
        )?;

        let events = stmt.query_map([process_name], |row| {
            let w_ts: i64 = row.get(17)?;
            let w_dur: i64 = row.get(18)?;
            let e_ts: i64 = row.get(20)?;
            let e_dur: i64 = row.get(21)?;

            Ok(EventModel {
                window: Some(WindowModel {
                    hwnd: row.get(0)?,
                    title: row.get(1)?,
                    class_name: row.get(2)?,
                    process_name: row.get(3)?,
                    process_path: row.get(4)?,
                    pid: row.get(5)?,
                    rect: Rect {
                        left: row.get(6)?,
                        top: row.get(7)?,
                        right: row.get(8)?,
                        bottom: row.get(9)?,
                        width: row.get(10)?,
                        height: row.get(11)?,
                    },
                    is_minimized: row.get(12)?,
                    is_maximized: row.get(13)?,
                    is_visible: row.get(14)?,
                    is_focused: row.get(15)?,
                    monitor_id: row.get(16)?,
                    timestamp: w_ts as u64,
                    duration: w_dur as u64,
                }),
                event_type: Self::event_type_from_i32(row.get(19)?),
                timestamp: e_ts as u64,
                duration: e_dur as u64,
            })
        })?;

        Ok(events.filter_map(Result::ok).collect())
    }

    /// Получить общее время активности по процессам
    pub fn get_total_time_by_process(&self) -> Result<Vec<(String, i64)>> {
        let mut stmt = self.conn.prepare(
            r#"
            SELECT w.process_name, SUM(e.duration)
            FROM events e
            JOIN window_activity w ON e.window_activity_id = w.id
            GROUP BY w.process_name
            ORDER BY SUM(e.duration) DESC
            "#,
        )?;

        let rows = stmt.query_map([], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, i64>(1)?))
        })?;

        Ok(rows.filter_map(Result::ok).collect())
    }

    /// Получить количество событий по типам
    pub fn get_events_count_by_type(&self) -> Result<Vec<(i32, i64)>> {
        let mut stmt = self.conn.prepare(
            r#"
            SELECT event_type, COUNT(*)
            FROM events
            GROUP BY event_type
            ORDER BY COUNT(*) DESC
            "#,
        )?;

        let rows = stmt.query_map([], |row| {
            Ok((row.get::<_, i32>(0)?, row.get::<_, i64>(1)?))
        })?;

        Ok(rows.filter_map(Result::ok).collect())
    }

    /// Получить последние N событий
    pub fn get_latest_events(&self, limit: i64) -> Result<Vec<EventModel>> {
        let mut stmt = self.conn.prepare(
            r#"
            SELECT 
                w.hwnd, w.title, w.class_name,
                w.process_name, w.process_path, w.pid,
                w.left, w.top, w.right, w.bottom, w.width, w.height,
                w.is_minimized, w.is_maximized, w.is_visible, w.is_focused,
                w.monitor_id, w.timestamp, w.duration,
                e.event_type, e.timestamp, e.duration
            FROM events e
            JOIN window_activity w ON e.window_activity_id = w.id
            ORDER BY e.timestamp DESC
            LIMIT ?1
            "#,
        )?;

        let events = stmt.query_map([limit], |row| {
            let w_ts: i64 = row.get(17)?;
            let w_dur: i64 = row.get(18)?;
            let e_ts: i64 = row.get(20)?;
            let e_dur: i64 = row.get(21)?;

            Ok(EventModel {
                window: Some(WindowModel {
                    hwnd: row.get(0)?,
                    title: row.get(1)?,
                    class_name: row.get(2)?,
                    process_name: row.get(3)?,
                    process_path: row.get(4)?,
                    pid: row.get(5)?,
                    rect: Rect {
                        left: row.get(6)?,
                        top: row.get(7)?,
                        right: row.get(8)?,
                        bottom: row.get(9)?,
                        width: row.get(10)?,
                        height: row.get(11)?,
                    },
                    is_minimized: row.get(12)?,
                    is_maximized: row.get(13)?,
                    is_visible: row.get(14)?,
                    is_focused: row.get(15)?,
                    monitor_id: row.get(16)?,
                    timestamp: w_ts as u64,
                    duration: w_dur as u64,
                }),
                event_type: Self::event_type_from_i32(row.get(19)?),
                timestamp: e_ts as u64,
                duration: e_dur as u64,
            })
        })?;

        Ok(events.filter_map(Result::ok).collect())
    }

    pub fn get_events_since(&self, from_ts: i64, limit: i64) -> Result<Vec<EventModel>> {
        let mut stmt = self.conn.prepare(
            r#"
            SELECT
                w.hwnd, w.title, w.class_name,
                w.process_name, w.process_path, w.pid,
                w.left, w.top, w.right, w.bottom, w.width, w.height,
                w.is_minimized, w.is_maximized, w.is_visible, w.is_focused,
                w.monitor_id, w.timestamp, w.duration,
                e.event_type, e.timestamp, e.duration
            FROM events e
            LEFT JOIN window_activity w ON e.window_activity_id = w.id
            WHERE e.timestamp >= ?1
            ORDER BY e.timestamp DESC
            LIMIT ?2
            "#,
        )?;

        let mut events: Vec<EventModel> = stmt
            .query_map([from_ts, limit], |row| Self::row_to_event(row))?
            .filter_map(Result::ok)
            .collect();

        events.reverse();
        Ok(events)
    }

    /// Удалить старые события (по временной метке)
    pub fn cleanup_old_events(&self, older_than_ts: i64) -> Result<usize> {
        let deleted = self.conn.execute(
            "DELETE FROM events WHERE timestamp < ?1",
            [older_than_ts],
        )?;

        Ok(deleted)
    }

    /// Получить количество всех событий
    pub fn get_events_count(&self) -> Result<i64> {
        let count: i64 = self.conn.query_row(
            "SELECT COUNT(*) FROM events",
            [],
            |row| row.get(0),
        )?;

        Ok(count)
    }

    /// Получить количество записей об окнах
    pub fn get_window_records_count(&self) -> Result<i64> {
        let count: i64 = self.conn.query_row(
            "SELECT COUNT(*) FROM window_activity",
            [],
            |row| row.get(0),
        )?;

        Ok(count)
    }

    /// Преобразовать i32 в EventType
    fn event_type_from_i32(value: i32) -> EventType {
        match value {
            0 => EventType::Idle,
            1 => EventType::WindowSwitch,
            2 => EventType::Keyboard,
            3 => EventType::Mouse,
            _ => EventType::Idle,
        }
    }
}

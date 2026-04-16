use rusqlite::{Connection, Result};
use std::sync::{Arc, Mutex};

use crate::core::WindowModel;

pub type Db = Arc<Mutex<WindowsDatabase>>;

pub struct WindowsDatabase {
    conn: Connection,
}

impl WindowsDatabase {
    /// Создание + инициализация БД
    pub fn new(path: &str) -> Self {
        let conn = Connection::open(path).expect("Failed to open DB");

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

                timestamp INTEGER NOT NULL
            );

            CREATE INDEX IF NOT EXISTS idx_window_time
            ON window_activity(timestamp);

            CREATE INDEX IF NOT EXISTS idx_window_process
            ON window_activity(process_name);
            "#,
        )?;

        Ok(())
    }

    /// Вставка окна в БД
    pub fn insert_window(&self, w: &WindowModel) -> Result<()> {
        self.conn.execute(
          r#"
          INSERT INTO window_activity (
              hwnd,
              title,
              class_name,
              process_name,
              process_path,
              pid,

              left,
              top,
              right,
              bottom,
              width,
              height,

              is_minimized,
              is_maximized,
              is_visible,
              is_focused,

              monitor_id,
              timestamp
          )
          VALUES (
              :hwnd,
              :title,
              :class_name,
              :process_name,
              :process_path,
              :pid,

              :left,
              :top,
              :right,
              :bottom,
              :width,
              :height,

              :is_minimized,
              :is_maximized,
              :is_visible,
              :is_focused,

              :monitor_id,
              :timestamp
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
          },
      )?;

        Ok(())
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
}
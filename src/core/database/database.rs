use std::time::Duration;

use rusqlite::{Connection, Result};
use std::sync::{Arc, Mutex};

use crate::{
    DB, core::{EventModel, EventType, GoalModel, GoalOrder, JobModel, MIGRATIONS, Rect, TagModel, WindowDesktop, WindowModel, WindowVariant}, lib::{extract_icon, extract_icon_events}
};
use crate::core::window::icon_file_name;

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

        // MIGRATIONS.to_latest(&mut db.conn)
        //     .expect("Failed to run migrations");


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
                icon_base64 TEXT,

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

        self.migrate_events_window_activity_nullable()?;


        Ok(())
    }

    fn ensure_column(
        &self,
        table: &str,
        column: &str,
        sql: &str,
    ) -> Result<()> {
        let mut stmt = self
            .conn
            .prepare(&format!("PRAGMA table_info({})", table))?;

        let cols = stmt
            .query_map([], |row| row.get::<_, String>(1))?
            .filter_map(std::result::Result::ok)
            .collect::<Vec<_>>();

        if !cols.iter().any(|c| c == column) {
            self.conn.execute(sql, [])?;
        }

        Ok(())
    }

    fn migrate_events_window_activity_nullable(&self) -> Result<()> {
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
                SELECT
                    id,
                    CASE
                        WHEN window_activity_id = -1 THEN NULL
                        ELSE window_activity_id
                    END,
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

    fn tags_json(tags: &[TagModel]) -> String {
        serde_json::to_string(tags).unwrap_or_else(|_| "[]".to_string())
    }

    fn parse_tags_json(raw: Option<String>) -> Vec<TagModel> {
        raw.and_then(|s| serde_json::from_str(&s).ok())
            .unwrap_or_default()
    }

    pub fn merge_tags(&mut self, tags: &[TagModel]) -> Result<usize> {
        let mut inserted = 0;
        for t in tags {
            let exists: bool = self.conn.query_row(
                "SELECT EXISTS(SELECT 1 FROM tag WHERE name = ?1)",
                [&t.name],
                |row| row.get(0),
            )?;
            if !exists {
                self.conn.execute(
                    "INSERT INTO tag (name, description, color) VALUES (?1, ?2, ?3)",
                    (&t.name, &t.description, &t.color),
                )?;
                inserted += 1;
            }
        }
        Ok(inserted)
    }

    pub fn get_window_tag(&self, process_name: String) -> Result<Vec<TagModel>> {
        let mut stmt = self.conn.prepare(
            r#"
            SELECT
                t.id, t.name, t.description, t.color
            FROM tag_to_window w
                JOIN tag t ON t.id = w.tag_id
            WHERE w.process_name = ?1
            "#,
        )?;

        let tags = stmt.query_map([process_name], |row| {
            let tag_id: i64 = row.get(0)?;
            let name: String = row.get(1)?;
            let description: String = row.get(2)?;
            let color: String = row.get(3)?;

            Ok(TagModel {
                id: Some(tag_id),
                name,
                description: Some(description),
                color,
            })
        })?;

        Ok(tags.filter_map(Result::ok).collect())
    }

    pub fn get_windows(&self) -> Result<Vec<(WindowModel, Vec<TagModel>)>> {
        let mut stmt = self.conn.prepare(
            r#"
            WITH ranked AS (
                SELECT
                    id,
                    hwnd,
                    title,
                    class_name,
                    icon_base64,
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
                    duration,
                    timestamp,
                    ROW_NUMBER() OVER (
                        PARTITION BY LOWER(TRIM(process_name))
                        ORDER BY timestamp DESC, id DESC
                    ) AS rn
                FROM window_activity
            )
            SELECT
                hwnd,
                title,
                class_name,
                icon_base64,
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
                duration,
                timestamp,
                id
            FROM ranked
            WHERE rn = 1
            ORDER BY timestamp DESC
            "#,
        )?;


        let windows = stmt.query_map([], |row| {
            let hwnd: i64 = row.get(0)?;
            let title: String = row.get(1)?;
            let class_name: String = row.get(2)?;
            let icon_base64: Option<String> = row.get(3)?;
            let process_name: String = row.get(4)?;
            let process_path: String = row.get(5)?;
            let pid: i32 = row.get(6)?;
            let left: i32 = row.get(7)?;
            let top: i32 = row.get(8)?;
            let right: i32 = row.get(9)?;
            let bottom: i32 = row.get(10)?;
            let width: i32 = row.get(11)?;
            let height: i32 = row.get(12)?;
            let is_minimized: i32 = row.get(13)?;
            let is_maximized: i32 = row.get(14)?;
            let is_visible: i32 = row.get(15)?;
            let is_focused: i32 = row.get(16)?;
            let monitor_id: Option<i32> = row.get(17)?;
            let duration: i64 = row.get(18)?;
            let timestamp: i64 = row.get(19)?;
            let id: i64 = row.get(20)?;

            let rect = Rect {
                left,
                top,
                right,
                bottom,
                width,
                height,
            };

            let tags = self.get_window_tag(process_name.clone()).unwrap_or_default();

            let icon_base64 = extract_icon(icon_base64.unwrap_or(String::from("")));

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
                    variant: WindowVariant::Desktop(WindowDesktop {}),

                    is_minimized: is_minimized != 0,
                    is_maximized: is_maximized != 0,
                    is_visible: is_visible != 0,
                    is_focused: is_focused != 0,
                    monitor_id: monitor_id.map(|v| v as u32),
                    timestamp: timestamp as u64,
                    duration: duration as u64,
                },
                tags,
            ))
        })?;



        Ok(windows.collect::<Result<Vec<_>>>()?)
    }

    pub fn get_windows_by_process(
        &self,
        process_name: String,
    ) -> Result<Vec<(WindowModel, Vec<TagModel>)>> {
        let mut stmt = self.conn.prepare(
            r#"
            SELECT
                hwnd,
                title,
                class_name,
                icon_base64,
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
                duration,
                timestamp,
                id
            FROM window_activity
            WHERE LOWER(TRIM(process_name)) = LOWER(TRIM(?1))
            ORDER BY timestamp DESC, rowid DESC
            "#,
        )?;

        let windows = stmt.query_map([process_name], |row| {
            let hwnd: i64 = row.get(0)?;
            let title: String = row.get(1)?;
            let class_name: String = row.get(2)?;
            let icon_base64: Option<String> = row.get(3)?;
            let process_name: String = row.get(4)?;
            let process_path: String = row.get(5)?;
            let pid: i32 = row.get(6)?;
            let left: i32 = row.get(7)?;
            let top: i32 = row.get(8)?;
            let right: i32 = row.get(9)?;
            let bottom: i32 = row.get(10)?;
            let width: i32 = row.get(11)?;
            let height: i32 = row.get(12)?;
            let is_minimized: i32 = row.get(13)?;
            let is_maximized: i32 = row.get(14)?;
            let is_visible: i32 = row.get(15)?;
            let is_focused: i32 = row.get(16)?;
            let monitor_id: Option<i32> = row.get(17)?;
            let duration: i64 = row.get(18)?;
            let timestamp: i64 = row.get(19)?;
            println!("Window: {:?}", title);
            let id: i64 = row.get(20)?;

            let rect = Rect {
                left,
                top,
                right,
                bottom,
                width,
                height,
            };

            let tags = self.get_window_tag(process_name.clone()).unwrap_or_default();

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
                    variant: WindowVariant::Desktop(WindowDesktop {}),
                    is_minimized: is_minimized != 0,
                    is_maximized: is_maximized != 0,
                    is_visible: is_visible != 0,
                    is_focused: is_focused != 0,
                    monitor_id: monitor_id.map(|v| v as u32),
                    timestamp: timestamp as u64,
                    duration: duration as u64,
                },
                tags,
            ))
        })?;

        Ok(windows.collect::<Result<Vec<_>>>()?)
    }


    pub fn delete_window(&mut self, process_name: String) -> Result<()> {
        println!("Delete window: {}", process_name);

        let windows = self.get_windows_by_process(process_name.clone())?;
        println!("Searches windows count {}", windows.len());

        let tx = self.conn.transaction()?;

        for (window, _) in &windows {
            if let Some(id) = window.id {
                println!("Delete events for window id {}", id);

                if let Err(err) = tx.execute(
                    r#"
                    DELETE FROM events
                    WHERE window_activity_id = ?1
                    "#,
                    [id],
                ) {
                    println!("Delete events failed for id {}: {:?}", id, err);
                    return Err(err);
                }
            } else {
                println!("Skip window without id");
            }
        }

        if let Err(err) = tx.execute(
            r#"
            DELETE FROM window_activity
            WHERE LOWER(TRIM(process_name)) = LOWER(TRIM(?1))
            "#,
            [process_name.as_str()],
        ) {
            println!("Delete windows failed: {:?}", err);
            return Err(err);
        }

        tx.commit()?;

        println!("Delete committed");

        Ok(())
    }

    pub fn get_tags(&self) -> Result<Vec<TagModel>> {
        let mut stmt = self.conn.prepare(
            r#"
            SELECT
                id, name, description, color
            FROM tag
            "#,
        )?;

        let tags = stmt.query_map([], |row| {
            let id: i64 = row.get(0)?;
            let name: String = row.get(1)?;
            let description: Option<String> = row.get(2)?;
            let color: String = row.get(3)?;

            Ok(TagModel {
                id: Some(id),
                name,
                description,
                color,
            })
        })?;

        Ok(tags.filter_map(Result::ok).collect())
    }

    pub fn add_tag_to_window(&self, tag_id: i64, process_name: String) -> Result<i64> {
        self.conn.execute(
            r#"
            INSERT INTO tag_to_window (
                tag_id, process_name
            )
            VALUES (?1, ?2)
            "#,
            (
                tag_id,
                process_name,
            ),
        )?;

        Ok(self.conn.last_insert_rowid())
    }

    pub fn get_goals(&self) -> Result<Vec<GoalModel>> {
        let mut stmt = self.conn.prepare(
            r#"
            SELECT
                id, name, description, ordering, timestamp,
                start_period_ts, end_period_ts, process, tags, completed
            FROM goals
            ORDER BY timestamp DESC
            "#,
        )?;

        let goals = stmt.query_map([], |row| {
            let tags_raw: Option<String> = row.get(8)?;
            Ok(GoalModel {
                id: Some(row.get(0)?),
                name: row.get(1)?,
                description: row.get(2)?,
                ordering: GoalOrder::from_i32(row.get(3)?),
                timestamp: row.get(4)?,
                start_period_ts: row.get(5)?,
                end_period_ts: row.get(6)?,
                process: row.get(7)?,
                tags: Self::parse_tags_json(tags_raw),
                completed: row.get::<_, i32>(9)? != 0,
            })
        })?;

        Ok(goals.filter_map(std::result::Result::ok).collect())
    }

    pub fn insert_goal(&mut self, goal: &GoalModel) -> Result<i64> {
        let tags = Self::tags_json(&goal.tags);
        self.conn.execute(
            r#"
            INSERT INTO goals (
                name, description, ordering, timestamp,
                start_period_ts, end_period_ts, process, tags, completed
            )
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)
            "#,
            (
                &goal.name,
                &goal.description,
                goal.ordering.as_i32(),
                goal.timestamp,
                goal.start_period_ts,
                goal.end_period_ts,
                &goal.process,
                tags,
                goal.completed as i32,
            ),
        )?;
        Ok(self.conn.last_insert_rowid())
    }

    pub fn update_goal(&self, goal: &GoalModel) -> Result<()> {
        let Some(id) = goal.id else {
            return Err(rusqlite::Error::QueryReturnedNoRows);
        };
        let tags = Self::tags_json(&goal.tags);
        self.conn.execute(
            r#"
            UPDATE goals SET
                name = ?1,
                description = ?2,
                ordering = ?3,
                timestamp = ?4,
                start_period_ts = ?5,
                end_period_ts = ?6,
                process = ?7,
                tags = ?8,
                completed = ?9
            WHERE id = ?10
            "#,
            (
                &goal.name,
                &goal.description,
                goal.ordering.as_i32(),
                goal.timestamp,
                goal.start_period_ts,
                goal.end_period_ts,
                &goal.process,
                tags,
                goal.completed as i32,
                id,
            ),
        )?;
        Ok(())
    }

    pub fn delete_goal(&self, id: i64) -> Result<()> {
        self.conn
            .execute("DELETE FROM goals WHERE id = ?1", [id])?;
        Ok(())
    }

    /// Вставка окна в БД
    pub fn insert_window(&self, w: &WindowModel) -> Result<i64> {
        let icon_ref = icon_file_name(&w.process_path);

        self.conn.execute(
            r#"
            INSERT INTO window_activity (
                hwnd, title, class_name,
                process_name, process_path, pid,
                left, top, right, bottom, width, height,
                is_minimized, is_maximized, is_visible, is_focused,
                monitor_id, timestamp, duration, icon_base64
            )
            VALUES (
                :hwnd, :title, :class_name,
                :process_name, :process_path, :pid,
                :left, :top, :right, :bottom, :width, :height,
                :is_minimized, :is_maximized, :is_visible, :is_focused,
                :monitor_id, :timestamp, :duration, :icon_base64
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
                ":icon_base64": icon_ref,
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

    pub fn insert_jobs(&mut self, jobs: &JobModel) -> Result<()> {
        let process_paths = jobs
            .proccess_path
            .iter()
            .fold(String::new(), |acc, e| format!("{acc},{e:?}"));
        let tags = Self::tags_json(&jobs.tags);
        self.conn.execute(
            r#"
            INSERT INTO jobs (
                name,
                description,
                def_start_ts,
                def_end_ts,
                start_ts,
                end_ts,
                proccess_path,
                tags,
                cron,
                color
            )
            VALUES (
                :name, :description, :def_start_ts,
                :def_end_ts, :start_ts, :end_ts,
                :proccess_path, :tags, :cron, :color
            )
            "#,
            rusqlite::named_params! {
                ":name": jobs.name,
                ":description": jobs.description,
                ":def_start_ts": jobs.def_start_ts,
                ":def_end_ts": jobs.def_end_ts,
                ":start_ts": jobs.start_ts,
                ":end_ts": jobs.end_ts,
                ":proccess_path": process_paths,
                ":tags": tags,
                ":cron": jobs.cron,
                ":color": jobs.color
            },
        )?;

        Ok(())
    }

    pub fn save_job(&mut self, job: &JobModel) -> Result<i64> {
        let process_paths = job
            .proccess_path
            .iter()
            .fold(String::new(), |acc, e| format!("{acc},{e:?}"));
        let tags = Self::tags_json(&job.tags);
        self.conn.execute(
            r#"
            INSERT INTO jobs (
                name,
                description,
                def_start_ts,
                def_end_ts,
                start_ts,
                end_ts,
                proccess_path,
                tags,
                cron,
                color
            )
            VALUES (
                :name, :description, :def_start_ts,
                :def_end_ts, :start_ts, :end_ts,
                :proccess_path, :tags, :cron, :color
            )
            "#,
            rusqlite::named_params! {
                ":name": job.name,
                ":description": job.description,
                ":def_start_ts": job.def_start_ts,
                ":def_end_ts": job.def_end_ts,
                ":start_ts": job.start_ts,
                ":end_ts": job.end_ts,
                ":proccess_path": process_paths,
                ":tags": tags,
                ":cron": job.cron,
                ":color": job.color
            },
        )?;

        Ok(self.conn.last_insert_rowid())
    }

    pub fn delete_job(&self, id: i64) -> Result<()> {
        self.conn
            .execute("DELETE FROM jobs WHERE id = ?1", [id])?;
        Ok(())
    }

    pub fn update_job(&self, job: &JobModel) -> Result<()> {
        let process_paths = job
            .proccess_path
            .iter()
            .fold(String::new(), |acc, e| format!("{acc},{e:?}"));
        let tags = Self::tags_json(&job.tags);
        let rows = if let Some(id) = job.id {
            self.conn.execute(
                r#"
                UPDATE jobs
                SET
                    name = :name,
                    description = :description,
                    def_start_ts = :def_start_ts,
                    def_end_ts = :def_end_ts,
                    start_ts = :start_ts,
                    end_ts = :end_ts,
                    proccess_path = :proccess_path,
                    tags = :tags,
                    cron = :cron,
                    color = :color
                WHERE id = :id
                "#,
                rusqlite::named_params! {
                    ":name": job.name,
                    ":description": job.description,
                    ":def_start_ts": job.def_start_ts,
                    ":def_end_ts": job.def_end_ts,
                    ":start_ts": job.start_ts,
                    ":end_ts": job.end_ts,
                    ":proccess_path": process_paths,
                    ":tags": tags,
                    ":cron": job.cron,
                    ":color": job.color,
                    ":id": id,
                },
            )?
        } else {
            self.conn.execute(
                r#"
                UPDATE jobs
                SET
                    name = :name,
                    description = :description,
                    def_start_ts = :def_start_ts,
                    def_end_ts = :def_end_ts,
                    start_ts = :start_ts,
                    end_ts = :end_ts,
                    proccess_path = :proccess_path,
                    tags = :tags,
                    cron = :cron,
                    color = :color
                WHERE name = :name_match AND start_ts = :start_ts_match AND end_ts = :end_ts_match
                "#,
                rusqlite::named_params! {
                    ":name": job.name,
                    ":description": job.description,
                    ":def_start_ts": job.def_start_ts,
                    ":def_end_ts": job.def_end_ts,
                    ":start_ts": job.start_ts,
                    ":end_ts": job.end_ts,
                    ":proccess_path": process_paths,
                    ":tags": tags,
                    ":cron": job.cron,
                    ":color": job.color,
                    ":name_match": job.name,
                    ":start_ts_match": job.start_ts,
                    ":end_ts_match": job.end_ts,
                },
            )?
        };

        if rows == 0 {
            return Err(rusqlite::Error::QueryReturnedNoRows);
        }

        Ok(())
    }

    pub fn get_jobs(&self) -> Result<Vec<JobModel>> {
        let mut stmt = self.conn.prepare(
            r#"
            SELECT
                id, name, description,
                def_start_ts, def_end_ts,
                start_ts, end_ts,
                proccess_path, tags, cron, color
            FROM jobs
            ORDER BY start_ts DESC
            "#,
        )?;

        let jobs = stmt.query_map([], |row| {
            let proccess_path_str: String = row.get(7)?;
            let proccess_path: Vec<Option<i64>> = proccess_path_str
                .split(',')
                .filter(|s| !s.is_empty() && *s != "None")
                .map(|s| s.trim().parse::<i64>().ok())
                .collect();
            let tags_raw: Option<String> = row.get(8)?;

            Ok(JobModel {
                id: Some(row.get(0)?),
                name: row.get(1)?,
                description: row.get(2)?,
                def_start_ts: row.get(3)?,
                def_end_ts: row.get(4)?,
                start_ts: row.get(5)?,
                end_ts: row.get(6)?,
                proccess_path,
                tags: Self::parse_tags_json(tags_raw),
                cron: row.get(9)?,
                color: row.get(10)?,
            })
        })?;

        Ok(jobs.filter_map(Result::ok).collect())
    }

    pub fn get_jobs_for_day(&self, _day_start_ts: i64, _day_end_ts: i64) -> Result<Vec<JobModel>> {
        let mut stmt = self.conn.prepare(
            r#"
            SELECT
                id, name, description,
                def_start_ts, def_end_ts,
                start_ts, end_ts,
                proccess_path, tags, cron, color
            FROM jobs
            ORDER BY start_ts ASC
            "#,
        )?;

        let jobs = stmt.query_map([], |row| {
            let proccess_path_str: String = row.get(7)?;
            let proccess_path: Vec<Option<i64>> = proccess_path_str
                .split(',')
                .filter(|s| !s.is_empty() && *s != "None")
                .map(|s| s.trim().parse::<i64>().ok())
                .collect();
            let tags_raw: Option<String> = row.get(8)?;

            Ok(JobModel {
                id: Some(row.get(0)?),
                name: row.get(1)?,
                description: row.get(2)?,
                def_start_ts: row.get(3)?,
                def_end_ts: row.get(4)?,
                start_ts: row.get(5)?,
                end_ts: row.get(6)?,
                proccess_path,
                tags: Self::parse_tags_json(tags_raw),
                cron: row.get(9)?,
                color: row.get(10)?,
            })
        })?;

        Ok(jobs.filter_map(Result::ok).collect())
    }

    pub fn insert_events(&mut self, events: &[EventModel]) -> Result<()> {
        if events.is_empty() {
            return Ok(());
        }

        let tx = self.conn.transaction()?;

        for event in events {
            let window_id: Option<i64> = match &event.window {
                Some(w) => {
                    let icon_ref = icon_file_name(&w.process_path);

                    tx.execute(
                        r#"
                        INSERT INTO window_activity (
                            hwnd, title, class_name,
                            process_name, process_path, pid,
                            left, top, right, bottom, width, height,
                            is_minimized, is_maximized, is_visible, is_focused,
                            monitor_id, timestamp, duration, icon_base64
                        )
                        VALUES (
                            :hwnd, :title, :class_name,
                            :process_name, :process_path, :pid,
                            :left, :top, :right, :bottom, :width, :height,
                            :is_minimized, :is_maximized, :is_visible, :is_focused,
                            :monitor_id, :timestamp, :duration, :icon_base64
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
                            ":icon_base64": icon_ref,
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
        let hwnd: Option<i64> = row.get(0).ok();

        let window = if hwnd.is_some() {
            Some(WindowModel {
                id: None,
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
                variant: WindowVariant::Desktop(WindowDesktop {}),
                is_minimized: row.get(12)?,
                is_maximized: row.get(13)?,
                is_visible: row.get(14)?,
                is_focused: row.get(15)?,
                monitor_id: row.get(16)?,
                timestamp: row.get::<_, i64>(17)? as u64,
                duration: row.get::<_, i64>(18)? as u64,
                icon_base64: row.get(22).ok(),
            })
        } else {
            None
        };

        Ok(EventModel {
            window,
            event_type: Self::event_type_from_i32(row.get(19)?),
            timestamp: row.get::<_, i64>(20)? as u64,
            duration: row.get::<_, i64>(21)? as u64,
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
            Ok((row.get::<_, String>(0)?, row.get::<_, i64>(1)?))
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
                e.event_type, e.timestamp, e.duration, w.icon_base64
            FROM events e
            LEFT JOIN window_activity w ON e.window_activity_id = w.id
            ORDER BY e.timestamp ASC
            "#,
        )?;

        let events = stmt.query_map([], |row| Self::row_to_event(row))?;

        let events = events.filter_map(Result::ok).collect();

        let events = extract_icon_events(events);

        Ok(events)
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
                e.event_type, e.timestamp, e.duration, w.icon_base64
            FROM events e
            LEFT JOIN window_activity w ON e.window_activity_id = w.id
            WHERE e.timestamp <= ?2
            AND (e.timestamp + e.duration) >= ?1
            ORDER BY e.timestamp ASC
            "#,
        )?;

        let events = stmt.query_map([from_ts, to_ts], |row| Self::row_to_event(row))?;

        let events = events.filter_map(Result::ok).collect();

        let events = extract_icon_events(events);

        Ok(events)
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
                e.event_type, e.timestamp, e.duration, w.icon_base64
            FROM events e
            LEFT JOIN window_activity w ON e.window_activity_id = w.id
            WHERE e.event_type = ?1
            ORDER BY e.timestamp ASC
            "#,
        )?;

        let events = stmt.query_map([type_i32], |row| Self::row_to_event(row))?;

        let events = events.filter_map(Result::ok).collect();

        let events = extract_icon_events(events);

        Ok(events)
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
                e.event_type, e.timestamp, e.duration, w.icon_base64, w.id
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
                    variant: WindowVariant::Desktop(WindowDesktop {}),

                    is_minimized: row.get(12)?,
                    is_maximized: row.get(13)?,
                    is_visible: row.get(14)?,
                    is_focused: row.get(15)?,
                    monitor_id: row.get(16)?,
                    icon_base64: row.get(22)?,
                    id: Some(row.get(23)?),
                    timestamp: w_ts as u64,
                    duration: w_dur as u64,
                }),
                event_type: Self::event_type_from_i32(row.get(19)?),
                timestamp: e_ts as u64,
                duration: e_dur as u64,
            })
        })?;

        let events = events.filter_map(Result::ok).collect();

        let events = extract_icon_events(events);

        Ok(events)
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

        let rows = stmt.query_map([], |row| Ok((row.get::<_, i32>(0)?, row.get::<_, i64>(1)?)))?;

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
                e.event_type, e.timestamp, e.duration, w.icon_base64, w.id
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
                    variant: WindowVariant::Desktop(WindowDesktop {}),
                    is_minimized: row.get(12)?,
                    is_maximized: row.get(13)?,
                    is_visible: row.get(14)?,
                    is_focused: row.get(15)?,
                    monitor_id: row.get(16)?,
                    timestamp: w_ts as u64,
                    duration: w_dur as u64,
                    icon_base64: row.get(22)?,
                    id: Some(row.get(23)?),
                }),
                event_type: Self::event_type_from_i32(row.get(19)?),
                timestamp: e_ts as u64,
                duration: e_dur as u64,
            })
        })?;

        let events = events.filter_map(Result::ok).collect();

        let events = extract_icon_events(events);

        Ok(events)
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
                e.event_type, e.timestamp, e.duration, w.icon_base64
            FROM events e
            LEFT JOIN window_activity w ON e.window_activity_id = w.id
            WHERE e.timestamp >= ?1
            ORDER BY e.timestamp DESC
            LIMIT ?2
            "#,
        )?;

        let mut events = Vec::new();

        let rows = stmt.query_map([from_ts, limit], |row| Self::row_to_event(row))?;

        for row in rows {
            match row {
                Ok(event) => events.push(event),
                Err(err) => println!("DB row error: {:?}", err),
            }
        }

        let mut events = extract_icon_events(events);

        events.reverse();
        Ok(events)
    }

    /// Удалить старые события (по временной метке)
    pub fn cleanup_old_events(&self, older_than_ts: i64) -> Result<usize> {
        let deleted = self
            .conn
            .execute("DELETE FROM events WHERE timestamp < ?1", [older_than_ts])?;

        Ok(deleted)
    }

    /// Получить количество всех событий
    pub fn get_events_count(&self) -> Result<i64> {
        let count: i64 = self
            .conn
            .query_row("SELECT COUNT(*) FROM events", [], |row| row.get(0))?;

        Ok(count)
    }

    /// Получить количество записей об окнах
    pub fn get_window_records_count(&self) -> Result<i64> {
        let count: i64 =
            self.conn
                .query_row("SELECT COUNT(*) FROM window_activity", [], |row| row.get(0))?;

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

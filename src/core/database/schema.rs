/// Database schema definitions and column names
/// This module documents the database structure and provides constants for column names

/// Window Activity table schema
pub mod window_activity {
    pub const TABLE: &str = "window_activity";
    pub const COL_ID: &str = "id";
    pub const COL_HWND: &str = "hwnd";
    pub const COL_TITLE: &str = "title";
    pub const COL_CLASS_NAME: &str = "class_name";
    pub const COL_ICON_BASE64: &str = "icon_base64";
    pub const COL_PROCESS_NAME: &str = "process_name";
    pub const COL_PROCESS_PATH: &str = "process_path";
    pub const COL_PID: &str = "pid";
    pub const COL_BROWSER_NAME: &str = "browser_name";
    pub const COL_BROWSER_URL: &str = "browser_url";
    pub const COL_LEFT: &str = "left";
    pub const COL_TOP: &str = "top";
    pub const COL_RIGHT: &str = "right";
    pub const COL_BOTTOM: &str = "bottom";
    pub const COL_WIDTH: &str = "width";
    pub const COL_HEIGHT: &str = "height";
    pub const COL_IS_MINIMIZED: &str = "is_minimized";
    pub const COL_IS_MAXIMIZED: &str = "is_maximized";
    pub const COL_IS_VISIBLE: &str = "is_visible";
    pub const COL_IS_FOCUSED: &str = "is_focused";
    pub const COL_MONITOR_ID: &str = "monitor_id";
    pub const COL_DURATION: &str = "duration";
    pub const COL_TIMESTAMP: &str = "timestamp";
    pub const COL_TAGS: &str = "tags";
    pub const COL_COLOR: &str = "color";
}

/// Events table schema
pub mod events {
    pub const TABLE: &str = "events";
    pub const COL_ID: &str = "id";
    pub const COL_WINDOW_ACTIVITY_ID: &str = "window_activity_id";
    pub const COL_EVENT_TYPE: &str = "event_type";
    pub const COL_TIMESTAMP: &str = "timestamp";
    pub const COL_DURATION: &str = "duration";
}

/// Jobs table schema
pub mod jobs {
    pub const TABLE: &str = "jobs";
    pub const COL_ID: &str = "id";
    pub const COL_NAME: &str = "name";
    pub const COL_DESCRIPTION: &str = "description";
    pub const COL_DEF_START_TS: &str = "def_start_ts";
    pub const COL_DEF_END_TS: &str = "def_end_ts";
    pub const COL_START_TS: &str = "start_ts";
    pub const COL_END_TS: &str = "end_ts";
    pub const COL_PROCESS_PATH: &str = "proccess_path";
    pub const COL_TAGS: &str = "tags";
    pub const COL_CRON: &str = "cron";
    pub const COL_COLOR: &str = "color";
    pub const COL_CREATED_AT: &str = "created_at";
}

/// Goals table schema
pub mod goals {
    pub const TABLE: &str = "goals";
    pub const COL_ID: &str = "id";
    pub const COL_NAME: &str = "name";
    pub const COL_DESCRIPTION: &str = "description";
    pub const COL_ORDERING: &str = "ordering";
    pub const COL_TIMESTAMP: &str = "timestamp";
    pub const COL_START_PERIOD_TS: &str = "start_period_ts";
    pub const COL_END_PERIOD_TS: &str = "end_period_ts";
    pub const COL_PROCESS: &str = "process";
    pub const COL_TAGS: &str = "tags";
    pub const COL_COMPLETED: &str = "completed";
}

/// Tag table schema
pub mod tag {
    pub const TABLE: &str = "tag";
    pub const COL_ID: &str = "id";
    pub const COL_NAME: &str = "name";
    pub const COL_DESCRIPTION: &str = "description";
    pub const COL_COLOR: &str = "color";
}

/// Tag to Window mapping table schema
pub mod tag_to_window {
    pub const TABLE: &str = "tag_to_window";
    pub const COL_TAG_ID: &str = "tag_id";
    pub const COL_PROCESS_NAME: &str = "process_name";
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_schema_constants() {
        assert_eq!(window_activity::TABLE, "window_activity");
        assert_eq!(window_activity::COL_COLOR, "color");
        assert_eq!(events::TABLE, "events");
        assert_eq!(jobs::TABLE, "jobs");
    }
}

use dioxus::prelude::*;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Theme {
    Light,
    Dark,
}

impl Theme {
    pub fn as_str(self) -> &'static str {
        match self {
            Theme::Light => "light",
            Theme::Dark => "dark",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SettingsProvider {
    pub theme: Signal<Theme>,
    pub enable_notifications: Signal<bool>,
    pub notification_delay_ms: Signal<u64>,
    pub tracker_report_interval_ms: Signal<u64>,
    pub db_flush_interval_ms: Signal<u64>,
    pub event_limit: Signal<i64>,
    pub compact_timeline: Signal<bool>,
    pub show_idle_events: Signal<bool>,
    pub auto_start_tracking: Signal<bool>,
}

impl Default for SettingsProvider {
    fn default() -> Self {
        Self {
            theme: Signal::new(Theme::Dark),
            enable_notifications: Signal::new(true),
            notification_delay_ms: Signal::new(1_500),
            tracker_report_interval_ms: Signal::new(5_000),
            db_flush_interval_ms: Signal::new(750),
            event_limit: Signal::new(1_000),
            compact_timeline: Signal::new(true),
            show_idle_events: Signal::new(true),
            auto_start_tracking: Signal::new(true),
        }
    }
}

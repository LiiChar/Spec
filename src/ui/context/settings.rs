use dioxus::prelude::*;
use serde::{Deserialize, Serialize};

use crate::core::{TagRule, TagRuleField};
use crate::lib::{load_settings, save_settings};

#[derive(Debug, Copy, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum Theme {
    #[default]
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
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "light" => Some(Theme::Light),
            "dark" => Some(Theme::Dark),
            _ => None,
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum Language {
    #[default]
    Russian,
    English,
}

impl Language {
    pub fn as_str(self) -> &'static str {
        match self {
            Language::Russian => "russian",
            Language::English => "english",
        }
    }
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "russian" => Some(Language::Russian),
            "english" => Some(Language::English),
            _ => None,
        }
    }
}


#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Settings {
    pub theme: Theme,
    pub language: Language,
    pub enable_notifications: bool,
    pub notification_delay_ms: u64,
    pub db_flush_interval_ms: u64,
    pub event_limit: i64,
    pub compact_timeline: bool,
    pub show_idle_events: bool,
    pub auto_start_tracking: bool,
    pub idle_threshold: u32,
    pub event_duration: u32,
    pub report_interval: u64,
    pub show_apps: bool,
    pub save_data: bool,
    pub work_start_hour: u32,
    pub work_end_hour: u32,
    pub show_browser_details: bool,
    pub tag_rules: Vec<TagRule>,
    pub soft_event: bool,
    pub show_tags: bool,
    pub type_tags: String,
    pub segment_height: u32,
    pub selected_segment_height: u32,
    pub show_current_time_line: bool,
    pub type_label: String,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            theme: Theme::Dark,
            language: Language::Russian,
            enable_notifications: true,
            notification_delay_ms: 1500,
            db_flush_interval_ms: 750,
            event_limit: 1000,
            compact_timeline: true,
            show_idle_events: true,
            auto_start_tracking: true,
            event_duration: 60,
            idle_threshold: 250,
            report_interval: 5000,
            show_apps: true,
            save_data: true,
            work_start_hour: 9,
            work_end_hour: 18,
            show_browser_details: true,
            tag_rules: vec![TagRule {
                field: TagRuleField::Process,
                pattern: r"chrome|edge|firefox|brave|opera".to_string(),
                tag: "Browser".to_string(),
                enabled: true,
            }],
            soft_event: false,
            show_tags: true,
            type_tags: "сircle".to_string(),
            segment_height: 80,
            selected_segment_height: 800,
            show_current_time_line: true,
            type_label: "full".to_string(),
        }
    }
}


#[derive(Clone)]
pub struct SettingsState {
    pub settings: Signal<Settings>,
}

impl SettingsState {
    pub fn save(&self) {
        let data = self.settings.read().clone();
        let _ = save_settings(&data);
    }

    fn mutate<F>(&mut self, f: F)
    where
        F: FnOnce(&mut Settings),
    {
        {
            // Захватываем блокировку на запись
            let mut write_guard = self.settings.write();
            // Передаём мутабельную ссылку на Settings в замыкание
            f(&mut *write_guard);
            // Здесь write_guard выходит из области видимости и разблокируется
        }
        // Сохраняем после того, как изменения применены
        self.save();
    }

    // ===== API =====

    pub fn set_theme(&mut self, theme: Theme) {
        self.mutate(|s| s.theme = theme);
    }

    pub fn set_language(&mut self, language: Language) {
        self.mutate(|s| s.language = language);
    }

    pub fn set_notifications(&mut self, v: bool) {
        self.mutate(|s| s.enable_notifications = v);
    }

    pub fn set_notification_delay(&mut self, v: u64) {
        self.mutate(|s| s.notification_delay_ms = v);
    }

    pub fn set_tracker_interval(&mut self, v: u64) {
        self.mutate(|s| s.report_interval = v);
    }

    pub fn set_db_flush_interval(&mut self, v: u64) {
        self.mutate(|s| s.db_flush_interval_ms = v);
    }

    pub fn set_event_limit(&mut self, v: i64) {
        self.mutate(|s| s.event_limit = v);
    }

    pub fn set_compact_timeline(&mut self, v: bool) {
        self.mutate(|s| s.compact_timeline = v);
    }

    pub fn set_show_idle_events(&mut self, v: bool) {
        self.mutate(|s| s.show_idle_events = v);
    }

    pub fn set_auto_start_tracking(&mut self, v: bool) {
        self.mutate(|s| s.auto_start_tracking = v);
    }

    pub fn set_save_data(&mut self, v: bool) {
        self.mutate(|s| s.save_data = v);
    }

    pub fn set_show_apps(&mut self, v: bool) {
        self.mutate(|s| s.show_apps = v);
    }

    pub fn set_work_hours(&mut self, start_hour: u32, end_hour: u32) {
        self.mutate(|s| {
            s.work_start_hour = start_hour.min(23);
            s.work_end_hour = end_hour.min(23);
        });
    }

    pub fn set_show_browser_details(&mut self, v: bool) {
        self.mutate(|s| s.show_browser_details = v);
    }

    pub fn set_tag_rules(&mut self, rules: Vec<TagRule>) {
        self.mutate(|s| s.tag_rules = rules);
    }

    pub fn set_event_duration(&mut self, v: u32) {
        self.mutate(|s| s.event_duration = v);
    }

    pub fn set_idle_threshold(&mut self, v: u32) {
        self.mutate(|s| s.idle_threshold = v);
    }

    pub fn set_soft_event(&mut self, v: bool) {
        self.mutate(|s| s.soft_event = v);
    }

    pub fn set_show_tags(&mut self, v: bool) {
        self.mutate(|s| s.show_tags = v);
    }

    pub fn set_type_tags<S: Into<String>>(&mut self, v: S) {
        self.mutate(|s| s.type_tags = v.into());
    }

    pub fn set_segment_height(&mut self, v: u32) {
        self.mutate(|s| s.segment_height = v);
    }

    pub fn set_selected_segment_height(&mut self, v: u32) {
        self.mutate(|s| s.selected_segment_height = v);
    }

    pub fn set_show_current_time_line(&mut self, v: bool) {
        self.mutate(|s| s.show_current_time_line = v);
    }

    pub fn set_type_label<S: Into<String>>(&mut self, v: S) {
        self.mutate(|s| s.type_label = v.into());
    }

    pub fn set_work_start_hour(&mut self, hour: u32) {
        self.mutate(|s| s.work_start_hour = hour.min(23));
    }

    pub fn set_work_end_hour(&mut self, hour: u32) {
        self.mutate(|s| s.work_end_hour = hour.min(23));
    }
}

pub fn provide_settings() {
    let initial = load_settings();

    use_context_provider(|| SettingsState {
        settings: Signal::new(initial),
    });
}

pub fn use_settings() -> SettingsState {
    use_context::<SettingsState>()
}

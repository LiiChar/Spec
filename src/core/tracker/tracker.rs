use regex::Regex;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

use crossbeam_channel::Sender;
use windows::Win32::System::SystemInformation::GetTickCount;
use windows::Win32::UI::Input::KeyboardAndMouse::GetLastInputInfo;
use windows::Win32::UI::Input::KeyboardAndMouse::LASTINPUTINFO;

use crate::core::{get_current_window, EventModel, EventType, TagRule, TagRuleField, WindowModel, with_database_mut};
use crate::lib::{current_ts, load_settings};
use crate::ui::context::Settings;

#[derive(Debug, Clone, PartialEq, Eq)]
struct Activity {
    window: Option<WindowModel>,
    idle: bool,
}

impl Activity {
    fn event_type(&self) -> EventType {
        if self.idle {
            EventType::Idle
        } else {
            EventType::WindowSwitch
        }
    }
}

#[derive(Debug)]
pub struct Tracker {
    current: Option<Activity>,
    start_time: Instant,
    pub stats: HashMap<String, Duration>,
    pub idle_threshold: u32,
    pub event_duration: u32,
    pub report_interval: Duration,
    pub settings_reload_at: Instant,
    pub rules: Vec<(TagRule, Regex)>,
    pub settings: Settings,
}

impl Tracker {
    pub fn new() -> Self {
        let settings = load_settings();
        let rules = compile_rules(&settings.tag_rules);
        Self {
            current: None,
            start_time: Instant::now(),
            report_interval: Duration::from_millis(settings.report_interval),
            stats: HashMap::new(),
            idle_threshold: settings.idle_threshold,
            event_duration: settings.event_duration,
            settings_reload_at: Instant::now() + Duration::from_secs(5),
            rules,
            settings,
        }
    }

    pub fn tick(&mut self, tx: &Sender<EventModel>) {
        self.maybe_reload_settings();

        let now = Instant::now();

        if let Some(current) = &self.current {
            let duration = now - self.start_time;
            if duration >= self.report_interval {
                self.send_event(current, duration, tx);
                self.start_time = now;
            }
        }
    }

    fn send_event(&self, activity: &Activity, duration: Duration, tx: &Sender<EventModel>) {
        if duration.as_millis() < self.event_duration as u128 {
            return;
        }

        let _ = tx.send(EventModel {
            window: activity.window.clone(),
            event_type: activity.event_type(),
            timestamp: current_ts() - duration.as_millis() as u64,
            duration: duration.as_millis() as u64,
        });
    }

    fn switch(&mut self, mut next: Activity, tx: &Sender<EventModel>) {
        self.maybe_reload_settings();
        self.apply_rules(&mut next);

        let now = Instant::now();

        if let Some(current) = &self.current {
            let duration = now - self.start_time;

            if duration.as_millis() > 0 {
                let key = self.key(current);
                *self.stats.entry(key).or_default() += duration;
                self.send_event(current, duration, tx);
            }
        }

        self.current = Some(next);
        self.start_time = now;
    }

    fn key(&self, activity: &Activity) -> String {
        let base = match &activity.window {
            Some(w) => format!("{}::{}", w.process_name, w.title),
            None => "UNKNOWN".into(),
        };

        if activity.idle {
            format!("{}::IDLE", base)
        } else {
            base
        }
    }

    pub fn finalize(&mut self, tx: &Sender<EventModel>) {
        if let Some(current) = &self.current {
            let duration = Instant::now() - self.start_time;
            if duration.as_millis() > 0 {
                let key = self.key(current);
                *self.stats.entry(key).or_default() += duration;
                self.send_event(current, duration, tx);
            }
        }

        self.current = None;
    }
}

fn is_user_idle(threshold_secs: u32) -> bool {
    unsafe {
        let mut info = LASTINPUTINFO {
            cbSize: std::mem::size_of::<LASTINPUTINFO>() as u32,
            dwTime: 0,
        };

        if !GetLastInputInfo(&mut info).as_bool() {
            return false;
        }

        let tick = GetTickCount();
        let idle_ms = tick - info.dwTime;

        idle_ms / 1000 > threshold_secs
    }
}

pub fn start_tracking(tx: Sender<EventModel>) {
    let tracker = Arc::new(Mutex::new(Tracker::new()));
    let running = Arc::new(Mutex::new(true));

    let t_tracker = tracker.clone();
    let t_running = running.clone();

    let interval = t_tracker.lock().unwrap().report_interval;
    let threshold = t_tracker.lock().unwrap().idle_threshold;

    thread::spawn(move || {
        let poll_interval = interval;

        while *t_running.lock().unwrap() {
            let idle = is_user_idle(threshold);
            let next_activity = match get_current_window(None) {
                Some(win) if !win.process_path.trim().is_empty() => Activity {
                    window: Some(win),
                    idle,
                },
                _ => {
                    continue;
                },
            };

            let mut tracker = t_tracker.lock().unwrap();
            match &tracker.current {
                Some(current) if same_activity(current, &next_activity) => tracker.tick(&tx),
                _ => tracker.switch(next_activity, &tx),
            }

            drop(tracker);
            thread::sleep(poll_interval);
        }

        let mut tracker = t_tracker.lock().unwrap();
        tracker.finalize(&tx);
    });
}

fn same_activity(a: &Activity, b: &Activity) -> bool {
    if a.idle != b.idle {
        return false;
    }

    match (&a.window, &b.window) {
        (Some(w1), Some(w2)) => {
            w1.title == w2.title
                && w1.process_name == w2.process_name
                && w1.variant == w2.variant
        }
        (None, None) => true,
        _ => false,
    }
}

fn compile_rules(rules: &[TagRule]) -> Vec<(TagRule, Regex)> {
    rules
        .iter()
        .filter_map(|rule| {
            if !rule.enabled || rule.pattern.trim().is_empty() {
                return None;
            }

            Regex::new(&rule.pattern)
                .ok()
                .map(|regex| (rule.clone(), regex))
        })
        .collect()
}

impl Tracker {
    fn maybe_reload_settings(&mut self) {
        let now = Instant::now();
        if now >= self.settings_reload_at {
            let settings = load_settings();
            if settings != self.settings {
                self.report_interval = Duration::from_millis(settings.report_interval);
                self.idle_threshold = settings.idle_threshold;
                self.event_duration = settings.event_duration;
                self.rules = compile_rules(&settings.tag_rules);
                self.settings = settings;
            }
            self.settings_reload_at = now + Duration::from_secs(5);
        }
    }

    fn apply_rules(&self, activity: &mut Activity) {
        if let Some(window) = activity.window.as_ref() {
            for (rule, regex) in &self.rules {
                let haystack = match rule.field {
                    TagRuleField::Process => window.process_name.as_str(),
                    TagRuleField::Title => window.title.as_str(),
                    TagRuleField::BrowserUrl => match &window.variant {
                        crate::core::WindowVariant::Browser(browser) => browser.url.as_str(),
                        _ => "",
                    },
                    TagRuleField::Any => &format!(
                            "{} {} {}",
                            window.process_name,
                            window.title,
                            match &window.variant {
                                crate::core::WindowVariant::Browser(browser) => browser.url.clone(),
                                _ => "".to_string(),
                            }
                        )
                };

                if regex.is_match(&haystack) {
                    let process_name = window.process_name.clone();
                    let tag_name = rule.tag.clone();
                    let _ = with_database_mut(|db| db.add_tag_to_window_if_missing(&tag_name, process_name.clone()));
                }
            }
        }
    }
}

pub fn stop_tracking(running: Arc<Mutex<bool>>) {
    let mut r = running.lock().unwrap();
    *r = false;
}

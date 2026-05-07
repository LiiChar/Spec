use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

use crossbeam_channel::Sender;
use windows::Win32::System::SystemInformation::GetTickCount;
use windows::Win32::UI::Input::KeyboardAndMouse::GetLastInputInfo;
use windows::Win32::UI::Input::KeyboardAndMouse::LASTINPUTINFO;

use crate::core::{get_current_window, EventModel, EventType, WindowModel};
use crate::lib::{current_ts, load_settings};

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
}

impl Tracker {
    pub fn new() -> Self {
        let settings = load_settings();
        Self {
            current: None,
            start_time: Instant::now(),
            report_interval: Duration::from_millis(settings.report_interval),
            stats: HashMap::new(),
            idle_threshold: settings.idle_threshold,
            event_duration: settings.event_duration,

        }
    }

    pub fn tick(&mut self, tx: &Sender<EventModel>) {
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

    fn switch(&mut self, next: Activity, tx: &Sender<EventModel>) {
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
                _ => Activity {
                    window: None,
                    idle: true,
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
        (Some(w1), Some(w2)) => w1.title == w2.title && w1.process_name == w2.process_name,
        (None, None) => true,
        _ => false,
    }
}

pub fn stop_tracking(running: Arc<Mutex<bool>>) {
    let mut r = running.lock().unwrap();
    *r = false;
}

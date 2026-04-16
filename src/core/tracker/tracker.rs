use std::collections::HashMap;
use std::sync::Mutex;
use std::time::{Duration, Instant};

use chrono::DateTime;
use crossbeam_channel::Sender;
use windows::Win32::Foundation::*;
use windows::Win32::UI::WindowsAndMessaging::*;
use windows::Win32::System::Threading::*;
use windows::Win32::UI::Accessibility::*;
use windows::Win32::System::SystemInformation::GetTickCount;
use windows::Win32::UI::Input::KeyboardAndMouse::*;

use crate::core::{EventModel, EventType, WindowModel, get_current_window};
use crate::lib::current_ts;

static mut TRACKER: Option<Mutex<Tracker>> = None;
static mut HOOK: Option<HWINEVENTHOOK> = None;
static mut RUNNING: bool = false;
static mut EVENT_TX: Option<Sender<EventModel>> = None;

#[derive(Debug)]
pub struct Tracker {
    pub current_app: Option<WindowModel>,
    pub start_time: Instant,
    pub stats: HashMap<WindowModel, Duration>,
}

impl Tracker {
    fn new() -> Self {
        Self {
            current_app: None,
            start_time: Instant::now(),
            stats: HashMap::new(),
        }
    }

    fn switch_app(&mut self, app: WindowModel) {
        let now = Instant::now();

        if let Some(current) = &self.current_app {
            let duration = now - self.start_time;
            *self.stats.entry(current.clone()).or_default() += duration;

            println!("Switched from {} after {:?}", current.title, duration);
        }

        self.current_app = Some(app.clone());
        self.start_time = now;

        println!("Now active: {:?}", app);
    }

    fn stop(&mut self) {
        println!("Tracker stopped. Finalizing state...");
        self.current_app = None;
    }
}

unsafe extern "system" fn win_event_proc(
    _hook: HWINEVENTHOOK,
    event: u32,
    hwnd: HWND,
    _id_object: i32,
    _id_child: i32,
    _event_thread: u32,
    _event_time: u32,
) {
    if event != EVENT_SYSTEM_FOREGROUND {
        return;
    }

    let win = match get_current_window(Some(hwnd)) {
        Some(w) => w,
        None => return
    };

    if let Some(tracker) = &TRACKER {
        let mut tracker = tracker.lock().unwrap();

        if is_user_idle(60) {
            // не считаем idle
            tracker.current_app = None;
            return;
        }

        if let Some(tx) = &EVENT_TX {
            let now = Instant::now();

            let duration = now
                .duration_since(tracker.start_time)
                .as_millis() as u64;

            if (duration as f64 / 1000.0) < 1.0 {
                return
            }

            if win.title.trim().len() == 0 {
                return
            }

            let _ = tx.send(EventModel {
                window: win.clone(),
                event_type: EventType::WindowSwitch,
                timestamp: current_ts(),
                duration,
            });
        }

        tracker.switch_app(win);
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
    unsafe {
        EVENT_TX = Some(tx);
        TRACKER = Some(Mutex::new(Tracker::new()));
        RUNNING = true;

        let hook = SetWinEventHook(
            EVENT_SYSTEM_FOREGROUND,
            EVENT_SYSTEM_FOREGROUND,
            None,
            Some(win_event_proc),
            0,
            0,
            WINEVENT_OUTOFCONTEXT,
        );

        if hook.0.is_null() {
            panic!("Failed to set hook");
        }

        HOOK = Some(hook);

        println!("Listening for window changes...");

        let mut msg = MSG::default();

        while RUNNING && GetMessageW(&mut msg, None, 0, 0).into() {
            if msg.message == WM_QUIT {
                break;
            }

            TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }

        println!("Message loop exited");
    }
}

pub fn stop_tracking() {
    unsafe {
        RUNNING = false;

        // 1. снимаем hook
        if let Some(hook) = HOOK {
            UnhookWinEvent(hook);
            HOOK = None;
        }

        // 2. закрываем tracker
        if let Some(t) = &TRACKER {
            let mut tracker = t.lock().unwrap();
            tracker.stop();
        }

        // 3. пробуждаем message loop (ВАЖНО!)
        PostThreadMessageW(
            GetCurrentThreadId(),
            WM_QUIT,
            WPARAM(0),
            LPARAM(0),
        );

        println!("Tracking fully stopped");
    }
}
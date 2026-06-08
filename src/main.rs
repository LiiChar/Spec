mod core;
mod lib;
mod ui;

use core::*;
use ui::*;
use dioxus::prelude::*;

use std::{path::Path, sync::{Arc, Mutex}, thread, time::Duration};

use crossbeam_channel::Receiver;
use dioxus::{
    desktop::{Config, WindowBuilder},
    logger::tracing::Level, prelude::{Asset, asset},
};
use once_cell::sync::Lazy;

use crate::lib::{Builder, load_icon, load_settings};

static RX: Lazy<Mutex<Option<Receiver<EventModel>>>> = Lazy::new(|| Mutex::new(None));
static DB: Lazy<Db> = Lazy::new(|| Arc::new(Mutex::new(Database::new(DATABASE_PATH))));

const DB_BATCH_SIZE: usize = 64;
const TRAY_ICON: Asset = asset!("/assets/tray_icon.png");

#[derive(Debug, Clone)]
struct AppArgs {
    autostart: bool,
    hidden: bool,
}

fn parse_args() -> AppArgs {
    let mut args = AppArgs {
        autostart: false,
        hidden: false,
    };

    for arg in std::env::args().skip(1) {
        match arg.as_str() {
            "--autostart" => args.autostart = true,
            "--hidden" => args.hidden = true,
            _ => {}
        }
    }

    args
}

fn main() {
    let args = parse_args();
    let exe = std::env::current_exe().unwrap();

    if let Some(dir) = exe.parent() {
        std::env::set_current_dir(dir).ok();
    }

    dioxus::logger::init(Level::INFO).unwrap();

    let settings = load_settings();

    let icon = load_icon(&Path::new(TRAY_ICON.bundled().absolute_source_path()));
    let window_config = WindowBuilder::new().with_decorations(false).with_window_icon(Some(icon)).with_visible(!args.hidden);

    if cfg!(debug_assertions) {
        println!("Debug starting application with args: {:?}", args);
    } else {
        if settings.auto_start_tracking {
            let builder = Builder::new().app_name("Spexe").arg("--autostart").arg("--hidden");
            let autolaunch = builder.build().unwrap();
            autolaunch.enable().unwrap();
        }
    }

    let (tx_forward, rx_forward) = crossbeam_channel::unbounded::<EventModel>();
    let (tx_db, rx_db) = crossbeam_channel::unbounded::<EventModel>();
    let (tx_ui, rx_ui) = crossbeam_channel::unbounded::<EventModel>();

    {
        DB.lock().unwrap();
        *RX.lock().unwrap() = Some(rx_ui);
    }

    thread::spawn(move || {
        while let Ok(event) = rx_forward.recv() {
            let _ = tx_db.send(event.clone());
            let _ = tx_ui.send(event);
        }
    });

    // получение событий и сохранение в базу
    thread::spawn(move || {
        let mut database = Database::new(DATABASE_PATH);

        let mut pending: Vec<EventModel> = Vec::with_capacity(DB_BATCH_SIZE);
        let mut current: Option<EventModel> = None;

        loop {
            match rx_db.recv_timeout(Duration::from_millis(settings.db_flush_interval_ms)) {
                Ok(event) => {
                    match current.as_mut() {
                        Some(active) => {
                            let same_window =
                                active.window.as_ref().map(|w| w.hwnd)
                                    == event.window.as_ref().map(|w| w.hwnd);

                            if same_window {
                                active.duration += event.duration;

                                if let (Some(active_w), Some(event_w)) =
                                    (active.window.as_mut(), event.window.as_ref())
                                {
                                    active_w.duration += event_w.duration;
                                    active_w.timestamp = event_w.timestamp;
                                }
                            } else {
                                pending.push(active.clone());
                                *active = event;
                            }
                        }

                        None => {
                            current = Some(event);
                        }
                    }

                    if pending.len() >= DB_BATCH_SIZE {
                        if let Err(err) = database.insert_events(&pending) {
                            eprintln!("Failed insert event batch: {:?}", err);
                        }
                        pending.clear();
                    }
                }

                Err(crossbeam_channel::RecvTimeoutError::Timeout) => {
                    if let Some(active) = current.take() {
                        pending.push(active);
                    }

                    if !pending.is_empty() {
                        if let Err(err) = database.insert_events(&pending) {
                            eprintln!("Failed insert event batch: {:?}", err);
                        }
                        pending.clear();
                    }
                }

                Err(crossbeam_channel::RecvTimeoutError::Disconnected) => {
                    if let Some(active) = current.take() {
                        pending.push(active);
                    }

                    if !pending.is_empty() {
                        if let Err(err) = database.insert_events(&pending) {
                            eprintln!("Failed insert event batch: {:?}", err);
                        }
                    }

                    break;
                }
            }
        }
    });

    // запуск трекинга и отправка событий в обработчики
    thread::spawn(move || {
        core::tracker::start_tracking(tx_forward);
    });
    
    // запуск ui части
    dioxus::LaunchBuilder::desktop()
        .with_cfg(Config::new().with_window(window_config))
        .launch(Root);
}

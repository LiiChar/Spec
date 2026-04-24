mod core;
mod ui;
mod lib;
mod config;

use core::*;
use ui::*;
use config::*;

use std::{
    sync::Mutex,
    thread,
    time::Duration,
};

use dioxus::{desktop::{Config, WindowBuilder}, logger::tracing::Level};
use crossbeam_channel::Receiver;
use once_cell::sync::Lazy;

static RX: Lazy<Mutex<Option<Receiver<EventModel>>>> =
    Lazy::new(|| Mutex::new(None));

const TRACKER_REPORT_INTERVAL_MS: u64 = 5_000;
const DB_FLUSH_INTERVAL_MS: u64 = 750;
const DB_BATCH_SIZE: usize = 64;



fn main() {
    dioxus::logger::init(Level::INFO).unwrap();

    let window_config = WindowBuilder::new().with_decorations(false);

    let (tx_forward, rx_forward) = crossbeam_channel::unbounded::<EventModel>();
    let (tx_db, rx_db) = crossbeam_channel::unbounded::<EventModel>();
    let (tx_ui, rx_ui) = crossbeam_channel::unbounded::<EventModel>();

    {
        *RX.lock().unwrap() = Some(rx_ui);
    }

    thread::spawn(move || {
        while let Ok(event) = rx_forward.recv() {
            let _ = tx_db.send(event.clone());
            let _ = tx_ui.send(event);
        }
    });

    thread::spawn(move || {
        let mut database = WindowsDatabase::new(DATABASE_PATH);
        let mut pending = Vec::with_capacity(DB_BATCH_SIZE);

        loop {
            match rx_db.recv_timeout(Duration::from_millis(DB_FLUSH_INTERVAL_MS)) {
                Ok(event) => {
                    pending.push(event);

                    if pending.len() >= DB_BATCH_SIZE {
                        if let Err(err) = database.insert_events(&pending) {
                            eprintln!("Failed insert event batch: {:?}", err);
                        }
                        pending.clear();
                    }
                }
                Err(crossbeam_channel::RecvTimeoutError::Timeout) => {
                    if !pending.is_empty() {
                        if let Err(err) = database.insert_events(&pending) {
                            eprintln!("Failed insert event batch: {:?}", err);
                        }
                        pending.clear();
                    }
                }
                Err(crossbeam_channel::RecvTimeoutError::Disconnected) => {
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

    thread::spawn(move || {
        core::tracker::start_tracking(tx_forward, TRACKER_REPORT_INTERVAL_MS);
    });

    dioxus::LaunchBuilder::desktop()
        .with_cfg(Config::new().with_window(window_config))
        .launch(Root);
}
        
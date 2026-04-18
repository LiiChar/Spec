mod core;
mod ui;
mod lib;
mod config;

use core::*;
use ui::*;
use lib::*;
use config::*;

use std::{sync::{Arc, Mutex}, thread};

use dioxus::{desktop::{Config, WindowBuilder}, logger::tracing::Level, prelude::*};
use crossbeam_channel::{unbounded, Sender, Receiver};
use once_cell::sync::Lazy;

static RX: Lazy<Mutex<Option<Receiver<EventModel>>>> =
    Lazy::new(|| Mutex::new(None));



fn main() {
    dioxus::logger::init(Level::INFO).unwrap();

    let window_config = WindowBuilder::new().with_decorations(false);

    let (tx, rx) = crossbeam_channel::unbounded::<EventModel>();

    {
        *RX.lock().unwrap() = Some(rx);
    }

    thread::spawn(move || {
        core::tracker::start_tracking(tx);
    });

    thread::spawn(move || {
        let database = WindowsDatabase::new(DATABASE_PATH);

        println!("Database path: {}", DATABASE_PATH);

        loop {
            std::thread::sleep(std::time::Duration::from_millis(100));

            if let Ok(rx_guard) = RX.lock() {
                if let Some(rx) = rx_guard.as_ref() {
                    while let Ok(event) = rx.try_recv() {
                        print!("{:?}", event.window.title);
                        database.insert_event(&event).expect("Failed insert event");
                    }
                }
            }
        }
    });

    dioxus::LaunchBuilder::desktop()
        .with_cfg(Config::new().with_window(window_config))
        .launch(Root);
}
        

mod core;
mod ui;
mod lib;

use core::*;
use ui::*;
use lib::*;

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

    dioxus::LaunchBuilder::desktop()
        .with_cfg(Config::new().with_window(window_config))
        .launch(Root);
}
        

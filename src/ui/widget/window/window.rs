use dioxus::prelude::*;

use crate::{
    core::with_database,
    ui::{use_app, window_list::WindowList},
};

#[component]
pub fn Windows() -> Element {
    let app = use_app();

    let _events = app.events.read();

    let windows = with_database(|db| {
        db.get_windows().unwrap_or_default()
    });

    rsx! {
        WindowList {
            windows
        }
    }
}
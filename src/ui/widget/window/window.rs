use dioxus::prelude::*;

use crate::{
    core::with_database, ui::{components::window::window_list::WindowList, context::use_app},
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
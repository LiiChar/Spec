use dioxus::prelude::*;

use crate::{
    core::with_database, ui::{components::window::window_list::WindowList, context::use_app},
};

#[component]
pub fn Windows() -> Element {
    let app = use_app();
    let mut windows = use_signal(Vec::new);

    // Trigger reload when events change (i.e., new data arrived)
    let events_len = app.events.read().len();
    use_effect(move || {
        let _ = events_len; // depend on events length
        spawn(async move {
            let result = tokio::task::spawn_blocking(|| {
                with_database(|db| db.get_windows().unwrap_or_default())
            }).await;
            if let Ok(w) = result {
                windows.set(w);
            }
        });
    });

    rsx! {
        WindowList { windows: windows() }
    }
}
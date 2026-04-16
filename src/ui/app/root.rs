use dioxus::{desktop::{Config, WindowBuilder}, logger::tracing::Level, prelude::*};

use crate::{RX, core::{EventModel, get_current_window}, ui::{Events, Layout, EventInfo}};

const MAIN_CSS: Asset = asset!("/assets/main.css");
const TAILWIND_CSS: Asset = asset!("/assets/tailwind.css");

#[component]
pub fn Root() -> Element {
    let mut events = use_signal(Vec::<EventModel>::new);

    let mut current_window = use_signal(|| {
            get_current_window(None)
            .expect("Failed get current window")
    });

    use_effect(move || {
        spawn(async move {
            loop {
                tokio::time::sleep(std::time::Duration::from_secs(1)).await;
                // не блокирует поток
                if let Ok(rx_guard) = RX.lock() {
                    if let Some(rx) = rx_guard.as_ref() {
                        while let Ok(event) = rx.try_recv() {
                            current_window.set(event.window.clone());
                            events.with_mut(|e| e.push(event));
                        }
                    }
                }
            }
        });
    });


    rsx! {
        document::Link { rel: "stylesheet", href: MAIN_CSS } 
        document::Link { rel: "stylesheet", href: TAILWIND_CSS }
        Layout { 
            div {
                class: "flex gap-4 h-full w-full p-4",
                EventInfo { event: current_window.read().clone() }
                Events { events: events.read().clone() }
            }
        }
        
    }
}


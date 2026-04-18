use dioxus::{desktop::{Config, WindowBuilder}, logger::tracing::Level, prelude::*};

use crate::{RX, config::DATABASE_PATH, core::{EventModel, WindowsDatabase, get_current_window}, ui::{EventInfo, Events, Layout}};

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
        let db = WindowsDatabase::new(DATABASE_PATH.clone());
        let evdb = db.get_all_events().expect("Failed get event from database");

        events.set(evdb);

        spawn(async move {
            loop {
                tokio::time::sleep(std::time::Duration::from_millis(100)).await;
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
                class: "flex gap-0 h-full w-full",
                
                // Основной контент
                div {
                    class: "flex-1 flex flex-col",
                    Events { events: events.read().clone() }
                }
            }
        }
        
    }
}



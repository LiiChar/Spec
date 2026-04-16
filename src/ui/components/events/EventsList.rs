use dioxus::prelude::*;

use crate::core::EventModel;

#[derive(Props, PartialEq, Clone)]
pub struct EventsListProps {
    events: Vec<EventModel>
}

#[component]
pub fn EventsList(props: EventsListProps) -> Element {
    rsx! {
        div {
            class: "flex flex-col",
            {props.events.iter().map(|event| {
                rsx! {
                    div {
                        class: "flex flex-row",

                        div {
                            class: "w-[100px] h-[100px] bg-blue-500 text-white rounded",
                            "{event.window.title}"
                        }
                    }
                }
            })}
        }
    }
}
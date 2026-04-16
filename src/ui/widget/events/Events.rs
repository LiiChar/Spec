use dioxus::prelude::*;

use crate::{core::EventModel, ui::{EventsList, EventsTimeline, EventsWeek, TimelineOrientation}};


#[component]
pub fn Events(events: Vec<EventModel>) -> Element {

    rsx! {
        div { 
            class: "flex flex-col gap-4 h-full",
            div {
                class: "h-full w-[50px]",
                EventsTimeline { events: events.clone(), orientation: TimelineOrientation::Vertical },
            }
        }
    }
}
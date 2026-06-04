use dioxus::prelude::*;

use crate::ui::Events;

#[component]
pub fn MainPage() -> Element {
    rsx! {
        div { class: "flex gap-0 h-full w-full relative",
            div { class: "flex-1 flex flex-col", Events {} }
        }
    }
}

use dioxus::prelude::*;

#[component]
pub fn Spinner() -> Element {
    rsx! {
        div { class: "w-5 h-5 border-2 border-current border-t-transparent rounded-full animate-spin" }
    }
}
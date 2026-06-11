use dioxus::prelude::*;

use crate::ui::{components::ui::{alerter::Alerter, toaster::Toaster}, context::{Theme, use_settings}, layout::{Header, Navigate}};


#[component]
pub fn Layout(children: Element) -> Element {
    let settings = use_settings().settings.read().clone();
    let theme_class = match settings.theme {
        Theme::Light => "",
        Theme::Dark => "dark",
    };

    rsx! {
        div { class: "flex flex-col w-full h-full bg-background",

            Header {}

            main { class: "mt-4 flex-1 {theme_class} p-2", {children} }

            Toaster {}
            Alerter {}
            Navigate {}
        }
    }
}

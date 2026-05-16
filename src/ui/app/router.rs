use crate::ui::{MainPage, SettingsPage, StatisticsPage, use_app};
use dioxus::prelude::*;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Page {
    Main,
    Settings,
    Statistics,
}

#[component]
pub fn Router() -> Element {
    let context = use_app();
    let page = context.page;

    rsx! {
        match *page.read() {
            Page::Main => rsx! { MainPage {} },
            Page::Settings => rsx! { SettingsPage {} },
            Page::Statistics => rsx! { StatisticsPage {} },
        }
    }
}

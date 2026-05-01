use dioxus::prelude::*;

use crate::ui::{Header, Navigate, Toaster};

#[component]
pub fn Layout(children: Element) -> Element {
    rsx! {
        div {
            class: "flex flex-col w-full h-full bg-background",

            Header {}

            main {
                class: "mt-4 flex-1 dark p-2",
                {children}
            }

            Toaster {}
            Navigate {}
        }
    }
}

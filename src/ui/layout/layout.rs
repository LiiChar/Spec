use dioxus::prelude::*;

use crate::ui::Header;

#[component]
pub fn Layout(children: Element) -> Element {
    rsx! {
        div {
            class: "flex flex-col w-full h-full",
            
            Header {},
            
            main {
                class: "mt-[16px] flex-1 dark",
                {children}
            }
        }
    }
}
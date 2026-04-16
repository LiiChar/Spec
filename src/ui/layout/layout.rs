use dioxus::prelude::*;

use crate::ui::Header;

#[component]
pub fn Layout(children: Element) -> Element {
    rsx! {
        Header {},
        main {
          class: "mt-[10px] dark",
          {children}
        }
    }
}
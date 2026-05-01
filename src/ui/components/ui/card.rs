use dioxus::prelude::*;

#[component]
pub fn Card(children: Element) -> Element {
    rsx! {
        div {
            class: "
                bg-background border border-border/40
                rounded-xl shadow-sm p-4
            ",
            {children}
        }
    }
}
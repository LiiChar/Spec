use dioxus::prelude::*;

#[component]
pub fn Badge(
    text: String,
    #[props(default = String::new())] class: String,
) -> Element {
    rsx! {
        span { class: "
                inline-flex items-center px-2 py-1 text-xs
                rounded-md bg-primary/10 text-primary
                {class}
            ",
            "{text}"
        }
    }
}
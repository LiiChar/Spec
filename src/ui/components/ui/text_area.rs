use dioxus::prelude::*;

#[component]
pub fn Textarea(
    value: String,
    #[props(optional)] oninput: Option<EventHandler<FormEvent>>,
    #[props(default = String::new())] class: String,
) -> Element {
    rsx! {
        textarea {
            class: "
                w-full min-h-[100px] rounded-md border border-border/40
                bg-background text-foreground p-3
                outline-none focus:border-primary
                transition-all {class}
            ",
            value: "{value}",

            oninput: move |e| {
                if let Some(handler) = &oninput {
                    handler.call(e);
                }
            }
        }
    }
}
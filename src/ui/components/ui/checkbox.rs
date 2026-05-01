use dioxus::prelude::*;

#[component]
pub fn Checkbox(
    checked: bool,
    #[props(optional)] onchange: Option<EventHandler<FormEvent>>,
) -> Element {
    rsx! {
        input {
            r#type: "checkbox",
            checked: checked,
            class: "w-4 h-4 accent-primary cursor-pointer",

            onchange: move |e| {
                if let Some(handler) = &onchange {
                    handler.call(e);
                }
            }
        }
    }
}
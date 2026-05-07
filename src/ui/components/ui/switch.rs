use dioxus::prelude::*;

#[derive(Props, PartialEq, Clone)]
pub struct SwitchProps {
    #[props(default = false)]
    pub checked: bool,
    #[props(default)]
    pub onclick: EventHandler<MouseEvent>,
}

#[component]
pub fn Switch(props: SwitchProps) -> Element {
    rsx! {
        div {
            class: format!("w-10 h-6 flex items-center rounded-full p-1 cursor-pointer transition-all border border-border/40 {}",
              if props.checked { "bg-primary" } else { "bg-muted" }
            ),
            onclick: move |e: MouseEvent| props.onclick.call(e),
            div {
                class: format!(" w-4 h-4 bg-white rounded-full transition-all {}",
                    if props.checked { "translate-x-4" } else { "" }
                )
            }
        }
    }
}

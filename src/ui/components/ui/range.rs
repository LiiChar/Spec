use dioxus::prelude::*;

pub struct RangeProps {
    pub min: u32,
    pub max: u32,
    pub value: u32,
    pub oninput: EventHandler<FormEvent>,
}

#[component]
pub fn Range(props: RangeProps) -> Element {
    let mut value = use_signal(props.value);

    let oninput = props.oninput.clone();

    rsx! {
        div {
            class: "relative flex h-6 w-full items-center",
            input {
                class: "h-full w-full appearance-none bg-transparent border-none focus:outline-none",
                type: "range",
                min: "{props.min}",
                max: "{props.max}",
                value: "{value}",
                oninput: move |evt| {
                    let value = evt.value().parse::<u32>().unwrap_or(props.value);
                    value.set(value);
                    oninput.call(evt);
                },
            }
            div {
                class: "absolute left-0 top-1/2 -translate-y-1/2 w-full h-1 bg-primary/20 rounded-md shadow-sm",
                style: format!("left: {}%;", value() * 100.0 / props.max()),
            }
        }
    }
}
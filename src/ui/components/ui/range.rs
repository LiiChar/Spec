use dioxus::prelude::*;

#[derive(Props, Clone, PartialEq)]
pub struct RangeProps {
    pub min: u32,
    pub max: u32,

    #[props(default = 1)]
    pub step: u32,

    pub value: u32,

    pub on_input: Callback<u32>,

    #[props(optional)]
    pub label: Option<String>,
}

#[component]
pub fn Range(props: RangeProps) -> Element {
    let percentage =
        ((props.value - props.min) as f64 / (props.max - props.min) as f64 * 100.0)
            .clamp(0.0, 100.0);

    rsx! {
        div {
            class: "flex flex-col gap-2 w-full",

            if let Some(label) = &props.label {
                div {
                    class: "flex justify-between items-center text-sm",

                    span { "{label}" }

                    span {
                        class: "font-medium text-primary",
                        "{props.value}"
                    }
                }
            }

            div {
                class: "relative h-6 flex items-center",

                // Фон
                div {
                    class: "absolute left-0 top-1/2 -translate-y-1/2 h-2 w-full rounded-full bg-primary/10"
                }

                // Заполнение
                div {
                    class: "absolute left-0 top-1/2 -translate-y-1/2 h-2 rounded-full bg-primary transition-all duration-150",
                    style: "width: {percentage}%;"
                }

                // Сам range
                input {
                    class: "
                        absolute inset-0
                        w-full
                        appearance-none
                        bg-transparent
                        cursor-pointer
                        transition-none
                        [&::-webkit-slider-thumb]:appearance-none
                        [&::-webkit-slider-thumb]:size-4
                        [&::-webkit-slider-thumb]:rounded-full
                        [&::-webkit-slider-thumb]:bg-primary
                        [&::-webkit-slider-thumb]:border-2
                        [&::-webkit-slider-thumb]:border-background
                        [&::-webkit-slider-thumb]:shadow-md
                        [&::-moz-range-thumb]:size-4
                        [&::-moz-range-thumb]:rounded-full
                        [&::-moz-range-thumb]:bg-primary
                        [&::-moz-range-thumb]:border-none
                    ",

                    r#type: "range",

                    min: "{props.min}",
                    max: "{props.max}",
                    step: "{props.step}",
                    value: "{props.value}",

                    oninput: move |evt| {
                        if let Ok(v) = evt.value().parse::<u32>() {
                            props.on_input.call(v);
                        }
                    }
                }
            }
        }
    }
}
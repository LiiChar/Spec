use dioxus::prelude::*;

#[derive(Props, Clone, PartialEq)]
pub struct ColorPickerProps {
    pub onselect: Callback<String>,
    #[props(default = vec![])]
    pub colors: Vec<String>,
    pub color: ReadSignal<String>,
}

#[component]
pub fn ColorPicker(props: ColorPickerProps) -> Element {
    let mut visible = use_signal(|| false);
    rsx! {
        div {
            class: "relative ",

            if props.colors.is_empty() {
                div {
                    class: "relative",
                    div {
                        class: "aspect-full w-10 h-10 rounded-full overflow-hidden",
                        div {
                            class: "w-full h-full",
                            style: "background-color: {props.color}",
                        }
                    }
                    input {
                        r#type: "color",
                        class: "absolute z-10 top-0 left-0 aspect-full w-10 h-10 rounded-full overflow-hidden opacity-0",
                        oninput: move |e| props.onselect.call(e.value().clone())
                    }
                }
            } else {
                div {
                    onclick: move |_| visible.set(true),
                    class: "aspect-full w-10 h-10 rounded-full overflow-hidden",
                    div {
                        class: "w-full h-full",
                        style: "background-color: {props.color}",
                    }
                }
                if visible.read().clone() {
                    div {
                        class: "absolute top-11 left-0 flex gap-1",

                        {
                            props.colors.clone().into_iter().map(|color| {
                                let color_clone = color.clone();

                                rsx! {
                                    div {
                                        key: "{color}",
                                        onclick: move |_| {
                                            props.onselect.call(color_clone.clone());
                                            visible.set(false);
                                        },
                                        class: "w-4 h-4 cursor-pointer rounded",
                                        style: "background-color: {color}",
                                    }
                                }
                            })
                        }
                    }
                }
            }
        }
    }
}

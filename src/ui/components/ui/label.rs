use dioxus::prelude::*;

#[derive(Props, PartialEq, Clone)]
pub struct LabelProps {
    #[props(default)]
    pub children: Element,

    #[props(optional)]
    pub for_id: Option<String>,

    #[props(default = String::new())]
    pub class: String,
}

#[component]
pub fn Label(props: LabelProps) -> Element {
    rsx! {
        label {
            class: "text-sm font-medium text-foreground {props.class}",
            r#for: props.for_id.clone().unwrap_or_default(),
            {props.children}
        }
    }
}
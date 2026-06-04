use dioxus::prelude::*;

#[derive(Props, PartialEq, Clone)]
pub struct FormFieldProps {
    pub label: Option<String>,
    pub error: Option<String>,
    pub helper: Option<String>,

    #[props(default)]
    pub children: Element,
}

#[component]
pub fn FormField(props: FormFieldProps) -> Element {
    rsx! {
        div { class: "flex flex-col gap-1 w-full",

            if let Some(label) = props.label {
                Label { "{label}" }
            }

            {props.children}

            if let Some(error) = props.error {
                span { class: "text-sm text-error", "{error}" }
            } else if let Some(helper) = props.helper {
                span { class: "text-sm text-muted-foreground", "{helper}" }
            }
        }
    }
}
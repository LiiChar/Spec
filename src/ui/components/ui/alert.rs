use dioxus::prelude::*;

#[derive(PartialEq, Clone)]
pub enum AlertVariant {
    Info,
    Success,
    Error,
    Warning,
}

#[component]
pub fn Alert(
    variant: AlertVariant,
    children: Element,
) -> Element {
    let class = match variant {
        AlertVariant::Info => "bg-info/10 border-info/40",
        AlertVariant::Success => "bg-success/10 border-success/40",
        AlertVariant::Error => "bg-error/10 border-error/40",
        AlertVariant::Warning => "bg-warning/10 border-warning/40",
    };

    rsx! {
        div {
            class: "border rounded-md p-3 text-sm {class}",
            {children}
        }
    }
}
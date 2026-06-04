use dioxus::prelude::*;

#[derive(PartialEq, Clone)]
pub enum InputVariant {
    Default,
    Primary,
    Secondary,
    Success,
    Error,
    Warning,
    Info,
}

#[derive(PartialEq, Clone)]
pub enum InputSize {
    Sm,
    Md,
    Lg,
}

#[derive(Props, PartialEq, Clone)]
pub struct InputProps {
    #[props(extends = GlobalAttributes, extends = input)]
    attributes: Vec<Attribute>,

    #[props(default = String::new())]
    pub class: String,

    #[props(default = InputVariant::Default)]
    pub variant: InputVariant,

    #[props(default = InputSize::Md)]
    pub size: InputSize,

    #[props(default)]
    pub value: String,

    #[props(optional)]
    pub oninput: Option<EventHandler<FormEvent>>,

    #[props(default = false)]
    pub disabled: bool,

    #[props(default = false)]
    pub loading: bool,

    #[props(optional)]
    pub placeholder: Option<String>,

    // слот под иконки
    #[props(optional)]
    pub left_icon: Option<Element>,

    #[props(optional)]
    pub right_icon: Option<Element>,
}

#[component]
pub fn Input(props: InputProps) -> Element {
    // variant
    let variant_class = match props.variant {
        InputVariant::Default => "bg-background border-border/40 focus:border-border",
        InputVariant::Primary => "bg-primary/10 border-primary/40 focus:border-primary",
        InputVariant::Secondary => "bg-secondary/10 border-secondary/40 focus:border-secondary",
        InputVariant::Success => "bg-success/10 border-success/40 focus:border-success",
        InputVariant::Error => "bg-error/10 border-error/40 focus:border-error",
        InputVariant::Warning => "bg-warning/10 border-warning/40 focus:border-warning",
        InputVariant::Info => "bg-info/10 border-info/40 focus:border-info",
    };

    // size
    let size_class = match props.size {
        InputSize::Sm => "h-8 text-sm px-2",
        InputSize::Md => "h-10 text-base px-3",
        InputSize::Lg => "h-12 text-lg px-4",
    };

    let disabled_class = if props.disabled {
        "opacity-50 cursor-not-allowed pointer-events-none"
    } else {
        ""
    };

    rsx! {
        div { class: "relative flex items-center w-full",

            // left icon
            if let Some(icon) = props.left_icon.clone() {
                div { class: "absolute left-3 flex items-center pointer-events-none",
                    {icon}
                }
            }

            input {
                class: "
                    w-full rounded-md shadow-sm transition-all outline-none
                    text-foreground
                    border
                    {variant_class}
                    {size_class}
                    {disabled_class}
                    {props.class}
                ",

                value: "{props.value}",
                disabled: props.disabled,

                placeholder: props.placeholder.clone().unwrap_or_default(),

                oninput: move |evt| {
                    if props.disabled || props.loading {
                        return;
                    }
                    if let Some(handler) = &props.oninput {
                        handler.call(evt);
                    }
                },
                ..props.attributes,
            }

            // right icon / loader
            if props.loading {
                div { class: "absolute right-3 w-4 h-4 border-2 border-current border-t-transparent rounded-full animate-spin" }
            } else if let Some(icon) = props.right_icon.clone() {
                div { class: "absolute right-3 flex items-center", {icon} }
            }
        }
    }
}

use dioxus::prelude::*;

#[derive(PartialEq, Clone)]
pub enum ButtonVariant {
    Default,
    Primary,
    Secondary,
    Success,
    Error,
    Warning,
    Info,
    Ghost,
    Icon,
}

#[derive(PartialEq, Clone)]
pub enum ButtonSize {
    Sm,
    Md,
    Lg,
}

#[derive(Props, PartialEq, Clone)]
pub struct ButtonProps {
    #[props(extends = GlobalAttributes, extends = button)]
    attributes: Vec<Attribute>,

    #[props(default = String::new())]
    pub class: String,

    #[props(default)]
    pub children: Element,

    #[props(default = ButtonVariant::Default)]
    pub variant: ButtonVariant,

    #[props(default = ButtonSize::Md)]
    pub size: ButtonSize,

    #[props(default = false)]
    pub disabled: bool,

    #[props(default = false)]
    pub loading: bool,

    #[props(optional)]
    pub onclick: Option<EventHandler<MouseEvent>>,
}

#[component]
pub fn Button(props: ButtonProps) -> Element {
    // variant styles
    let variant_class = match props.variant {
        ButtonVariant::Default => "bg-background hover:bg-background/80 text-foreground border border-border/40 hover:border-border/60",
        ButtonVariant::Primary => "bg-primary hover:bg-primary/80 text-foreground border border-primary/40 hover:border-primary/60",
        ButtonVariant::Secondary => "bg-secondary hover:bg-secondary/80 text-foreground border border-secondary/40 hover:border-secondary/60",
        ButtonVariant::Success => "bg-success hover:bg-success/80 text-foreground border border-success/40 hover:border-success/60",
        ButtonVariant::Error => "bg-error hover:bg-error/80 text-foreground border border-error/40 hover:border-error/60",
        ButtonVariant::Warning => "bg-warning hover:bg-warning/80 text-foreground border border-warning/40 hover:border-warning/60",
        ButtonVariant::Info => "bg-info hover:bg-info/80 text-foreground border border-info/40 hover:border-info/60",
        ButtonVariant::Ghost => "bg-transparent hover:bg-foreground/5 text-foreground border border-transparent",
        ButtonVariant::Icon => "bg-background/40 text-foreground border border-transparent aspect-square",
    };

    // size styles
    let size_class = match props.size {
        ButtonSize::Sm => "px-2 py-1 text-sm",
        ButtonSize::Md => "px-3 py-2 text-base",
        ButtonSize::Lg => "px-4 py-3 text-lg",
    };

    let disabled_class = if props.disabled {
        "opacity-50 cursor-not-allowed pointer-events-none"
    } else {
        "cursor-pointer"
    };

    let loading_class = if props.loading {
        "relative text-transparent"
    } else {
        ""
    };

    rsx! {
        button {
            class: "
                inline-flex items-center justify-center gap-2
                rounded-md shadow-sm transition-all
                {variant_class}
                {size_class}
                {disabled_class}
                {loading_class}
                {props.class}
            ",
            disabled: props.disabled,

            onclick: move |evt| {
                if props.disabled || props.loading {
                    return;
                }
                if let Some(handler) = &props.onclick {
                    handler.call(evt);
                }
            },

            ..props.attributes,

            // loader
            if props.loading {
                span {
                    class: "absolute w-4 h-4 border-2 border-current border-t-transparent rounded-full animate-spin"
                }
            }

            {props.children}
        }
    }
}

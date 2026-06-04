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
        ButtonVariant::Default => {
            "bg-background/60 text-foreground border border-border/60 hover:bg-background/60 hover:border-border"
        }

        ButtonVariant::Primary => {
            "bg-primary/20 text-foreground border border-blue-600 hover:bg-primary/40 hover:border-blue-700"
        }

        ButtonVariant::Secondary => {
            "bg-secondary/60 text-foreground border border-zinc-800 hover:bg-zinc-700 hover:border-zinc-700"
        }

        ButtonVariant::Success => {
            "bg-emerald-600 text-foreground border border-emerald-600 hover:bg-emerald-700 hover:border-emerald-700"
        }

        ButtonVariant::Error => {
            "bg-red-600 text-foreground border border-red-600 hover:bg-red-700 hover:border-red-700"
        }

        ButtonVariant::Warning => {
            "bg-amber-500 text-black border border-amber-500 hover:bg-amber-600 hover:border-amber-600"
        }

        ButtonVariant::Info => {
            "bg-cyan-600 text-foreground border border-cyan-600 hover:bg-cyan-700 hover:border-cyan-700"
        }

        ButtonVariant::Ghost => {
            "bg-transparent text-zinc-700 border border-transparent hover:bg-zinc-100"
        }

        ButtonVariant::Icon => {
            "bg-transparent text-zinc-600 border border-transparent hover:bg-zinc-100 hover:text-zinc-900 aspect-square"
        }
    };

    // size styles
    let size_class = match props.size {
        ButtonSize::Sm => "px-2 py-1 text-sm",
        ButtonSize::Md => "px-2.5 py-1.5 text-base",
        ButtonSize::Lg => "px-3 py-2 text-lg",
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
                span { class: "absolute w-4 h-4 border-2 border-current border-t-transparent rounded-full animate-spin" }
            }

            {props.children}
        }
    }
}

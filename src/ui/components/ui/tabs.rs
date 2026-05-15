use dioxus::prelude::*;

#[derive(PartialEq, Clone)]
pub enum TabsVariant {
    Default,
    Rounded,
}

#[derive(PartialEq, Clone)]
pub enum TabsOrientation {
    Horizontal,
    Vertical,
}

#[derive(PartialEq, Clone)]
pub struct TabsContext {
    pub value: Signal<String>,
    pub variant: TabsVariant,
}

#[derive(Props, PartialEq, Clone)]
pub struct ChildrenProps {
    pub children: Element,
    #[props(default = String::new())]
    pub class: String,
}

#[derive(Props, PartialEq, Clone)]
pub struct TabsProps {
    pub value: String,
    pub children: Element,
    #[props(default = TabsVariant::Default)]
    pub variant: TabsVariant,
    #[props(default = TabsOrientation::Horizontal)]
    pub orientation: TabsOrientation,
}

#[derive(Props, PartialEq, Clone)]
pub struct TabsTriggerProps {
    pub value: String,
    pub children: Element,
    #[props(default = String::new())]
    pub class: String,
}

#[component]
pub fn Tabs(props: TabsProps) -> Element {
    let state = use_signal(|| props.value);

    use_context_provider(|| TabsContext {
        value: state,
        variant: props.variant,
    });

    rsx! { div  { {props.children} } }
}

#[component]
pub fn TabsList(props: ChildrenProps) -> Element {
    let ctx = use_context::<TabsContext>();

    let class = match ctx.variant {
        TabsVariant::Default => "border-b border-border/40",
        TabsVariant::Rounded => {
            "border border-border/40 rounded-lg bg-secondary/40 backdrop-blur-md"
        }
    };

    rsx! {
        div { class: "flex transition-all overflow-hidden w-min {props.class} {class}", {props.children} }
    }
}

#[component]
pub fn TabsTrigger(props: TabsTriggerProps) -> Element {
    let mut ctx = use_context::<TabsContext>();

    let active = (ctx.value)() == props.value;

    let class = match ctx.variant {
        TabsVariant::Default => format!("px-2 py-0.5 text-sm border-b-2 {} {}", if active { "border-primary text-primary" } else { "border-transparent" }, props.class),
        TabsVariant::Rounded => format!("px-1.5 py-1 text-sm border border-transparent rounded-none last:rounded-l-none first:rounded-r-none hover:bg-secondary/30 {} {}", if active { "border-primary text-primary bg-secondary/40" } else { "bg-transparent" }, props.class),
    };

    rsx! {
        button {
            class: format!("cursor-pointer {}",
                class
            ),

            onclick: move |_| ctx.value.set(props.value.clone()),

            {props.children}
        }
    }
}

#[component]
pub fn TabsContent(props: TabsProps) -> Element {
    let ctx = use_context::<TabsContext>();

    if (ctx.value)() != props.value {
        return rsx! {};
    }

    rsx! {
        {props.children}
    }
}

use chrono::format;
use dioxus::prelude::*;
use dioxus_free_icons::icons::ld_icons::{LdChevronDown, LdChevronUp};
use dioxus_free_icons::Icon;

#[derive(Clone)]
pub struct SelectContext {
    pub open: Signal<bool>,
    pub title: Signal<String>,
    pub value: Signal<String>,
    pub cb: Callback<String>
}

#[component]
pub fn Select(
    value: String,
    children: Element,
    onchange: Callback<String>
) -> Element {
    let mut open = use_signal(|| false);
    let selected = use_signal(|| value.clone());
    let title = use_signal(|| value.clone());

    use_context_provider(|| SelectContext {
        title,
        open,
        value: selected,
        cb: onchange
    });

    rsx! { 
        div { 
            class: "relative group outline-none border-none",
            role: "button",
            tabindex: "0",
            onblur: move |_| open.set(false),
            onkeydown: move |evt| {
                let temp_open = open.read().clone();
                if evt.key() == Key::Escape {
                    open.set(false);
                }
                if evt.key() == Key::Enter {
                    open.set(!temp_open);
                }
            },
            {children}
        } 
    }
}

#[derive(Props, PartialEq, Clone)]
pub struct SelectTriggerProps {
    #[props(default = String::new())]
    class: String, 
}

#[component]
pub fn SelectTrigger(props: SelectTriggerProps) -> Element {
    let mut ctx = use_context::<SelectContext>();
    let is_open = ctx.open.read().clone();
    
    rsx! {
        div {
            class: format!("relative rounded-md border border-border/40 bg-background px-3 py-2 text-foreground group-focus:border-primary outline-none shadow select-none pr-10 {}", props.class),
            onclick: move |_| ctx.open.set(!is_open),
            div {
                class: "absolute right-2 top-1/2 -translate-y-1/2 text-xs",
                Icon { icon: LdChevronUp, class: format!("transition-all {}", if is_open { "rotate-180" } else { "" }) }
            }
            "{ctx.title}"
        }
    }
}

#[derive(Props, PartialEq, Clone)]
pub struct SelectContentProps {
    children: Element,
    #[props(default = String::new())]
    class: String,
}

#[component]
pub fn SelectContent(props: SelectContentProps) -> Element {
    let ctx = use_context::<SelectContext>();

    rsx! {
        div {
            class: "absolute z-1 mt-1 w-full rounded-md border border-border/40 bg-background  text-foreground outline-none focus:border-primary shadow select-none {props.class}",
            style: format!("{}", 
                if ctx.open.read().clone() {
                    "max-height: inherit; overflow: auto; margin-top: 2px; opacity: 1;"
                } else {
                    "max-height: 0; overflow: hidden; margin-top: 0; opacity: 0;"
                }
            ),
            {props.children}
        }
    }
}

#[derive(Props, PartialEq, Clone)]
pub struct SelectItemProps {
    value: String, 
    #[props(optional)]
    title: Option<String>, 
    children: Element,
    #[props(default = String::new())]
    class: String,
}

#[component]
pub fn SelectItem(props: SelectItemProps) -> Element {
    let mut ctx = use_context::<SelectContext>();
    let title = props.title.clone().unwrap_or(props.value.clone()).clone();

    let o_value = ctx.value.read().clone();
    let o_title = props.title.clone().unwrap_or(props.value.clone()).clone();
    let o_ctx_value = props.value.clone();

    
    use_future(move || {
        let ctx_value = o_value.clone();
        let value = o_ctx_value.clone();
        let f_title = o_title.clone();
        async move {
            if ctx_value == value {
                ctx.title.set(f_title);
            };
        }
    });

    rsx! {
        div {
            class: format!("px-3 py-2 hover:bg-foreground/5 cursor-pointer {} {}",
                if ctx.value.read().clone() == props.value.clone() {
                    "text-primary"
                } else {
                    ""
                },
                props.class
            ),
            onclick: move |_| {
                let t = title.clone();
                ctx.value.set(props.value.clone());
                ctx.title.set(t);
                ctx.open.set(false);
                ctx.cb.call(props.value.clone());
            },

            {props.children}
        }
    }
}
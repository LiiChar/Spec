use dioxus::prelude::*;

#[derive(Clone)]
pub struct DropdownContext {
    pub open: Signal<bool>,
}

#[component]
pub fn Dropdown(children: Element) -> Element {
    let open = use_signal(|| false);

    use_context_provider(|| DropdownContext { open });

    rsx! {
        div { class: "relative inline-block", {children} }

    }
}

#[component]
pub fn DropdownTrigger(children: Element) -> Element {
    let ctx = use_context::<DropdownContext>();
    let mut is_open = ctx.open.clone();
    let open = ctx.open.read().clone();
    rsx! {
        div { onclick: move |_| is_open.set(!open), class: "cursor-pointer", {children} }
    }
}

#[component]
pub fn DropdownContent(children: Element) -> Element {
    let ctx = use_context::<DropdownContext>();

    if !ctx.open.read().clone() {
        return rsx! {};
    }

    rsx! {
        div { class: "
                absolute mt-2 w-48 bg-background border border-border/40
                rounded-md shadow-md p-1 z-50
            ",
            {children}
        }
    }
}

#[component]
pub fn DropdownItem(
    children: Element,
    #[props(optional)] onclick: Option<EventHandler<MouseEvent>>,
) -> Element {
    let ctx = use_context::<DropdownContext>();
    let mut is_open = ctx.open.clone();


    rsx! {
        div {
            class: "px-3 py-2 hover:bg-foreground/5 rounded cursor-pointer",

            onclick: move |e| {
                is_open.set(false);
                if let Some(handler) = &onclick {
                    handler.call(e);
                }
            },

            {children}
        }
    }
}
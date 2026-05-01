use dioxus::prelude::*;

#[derive(Clone)]
pub struct DropdownContext {
    pub open: Signal<bool>,
}

#[component]
pub fn Dropdown(children: Element) -> Element {
    let open = use_signal(|| false);

    use_context_provider(|| DropdownContext { open });

    rsx! { div { class: "relative inline-block", {children} } }
}

#[component]
pub fn DropdownTrigger(children: Element) -> Element {
    let ctx = use_context::<DropdownContext>();

    rsx! {
        div {
            onclick: move |_| ctx.open.set(!ctx.open()),
            class: "cursor-pointer",
            {children}
        }
    }
}

#[component]
pub fn DropdownContent(children: Element) -> Element {
    let ctx = use_context::<DropdownContext>();

    if !ctx.open() {
        return rsx! {};
    }

    rsx! {
        div {
            class: "
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

    rsx! {
        div {
            class: "px-3 py-2 hover:bg-foreground/5 rounded cursor-pointer",

            onclick: move |e| {
                ctx.open.set(false);
                if let Some(handler) = &onclick {
                    handler.call(e);
                }
            },

            {children}
        }
    }
}
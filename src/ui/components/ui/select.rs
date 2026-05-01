use dioxus::prelude::*;

#[derive(Clone)]
pub struct SelectContext {
    pub open: Signal<bool>,
    pub value: Signal<String>,
}

#[component]
pub fn Select(
    value: String,
    children: Element,
) -> Element {
    let open = use_signal(|| false);
    let selected = use_signal(|| value);

    use_context_provider(|| SelectContext {
        open,
        value: selected,
    });

    rsx! { div { class: "relative w-48", {children} } }
}

#[component]
pub fn SelectTrigger() -> Element {
    let ctx = use_context::<SelectContext>();

    rsx! {
        div {
            class: "border px-3 py-2 rounded cursor-pointer",
            onclick: move |_| ctx.open.set(!ctx.open()),

            "{ctx.value()}"
        }
    }
}

#[component]
pub fn SelectContent(children: Element) -> Element {
    let ctx = use_context::<SelectContext>();

    if !ctx.open() {
        return rsx! {};
    }

    rsx! {
        div {
            class: "absolute mt-1 w-full border bg-background rounded shadow",

            {children}
        }
    }
}

#[component]
pub fn SelectItem(value: String, children: Element) -> Element {
    let ctx = use_context::<SelectContext>();

    rsx! {
        div {
            class: "px-3 py-2 hover:bg-foreground/5 cursor-pointer",

            onclick: move |_| {
                ctx.value.set(value.clone());
                ctx.open.set(false);
            },

            {children}
        }
    }
}
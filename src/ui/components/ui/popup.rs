use dioxus::prelude::*;
use web_sys::{window, HtmlElement};

#[derive(Clone)]
pub struct PopupContext {
    pub open: Signal<bool>,
}

#[derive(Props, PartialEq, Clone)]
pub struct PopupProps {
    pub open: bool,
    #[props(optional)]
    pub onclose: Option<EventHandler<()>>,
    pub children: Element,
}

#[component]
pub fn Popup(props: PopupProps) -> Element {
    let is_open = use_signal(|| props.open);
    let is_rendered = use_signal(|| props.open);

    // sync controlled
    use_effect(move || {
        if props.open {
            is_rendered.set(true);
            is_open.set(true);
        } else {
            // запускаем exit animation
            is_open.set(false);

            let is_rendered = is_rendered.clone();
            spawn(async move {
                gloo_timers::future::TimeoutFuture::new(200).await;
                is_rendered.set(false);
            });
        }
    });

    if !is_rendered() {
        return rsx! {};
    }

    use_context_provider(|| PopupContext {
        open: is_open,
    });

    // scroll lock
    use_effect(move || {
        let body = window().unwrap().document().unwrap().body().unwrap();
        if is_open() {
            body.style().set_property("overflow", "hidden").ok();
        } else {
            body.style().set_property("overflow", "auto").ok();
        }
    });

    // ESC + focus trap
    use_effect(move || {
        if !is_open() {
            return;
        }

        let doc = window().unwrap().document().unwrap();

        let handler = Closure::wrap(Box::new(move |e: web_sys::KeyboardEvent| {
            // ESC
            if e.key() == "Escape" {
                if let Some(cb) = &props.onclose {
                    cb.call(());
                }
            }

            // TAB trap
            if e.key() == "Tab" {
                let focusable = doc
                    .query_selector_all("button, [href], input, textarea, select, [tabindex]:not([tabindex='-1'])")
                    .unwrap();

                if focusable.length() == 0 {
                    return;
                }

                let first = focusable.get(0).unwrap();
                let last = focusable.get(focusable.length() - 1).unwrap();

                let active = doc.active_element();

                if e.shift_key() {
                    if active == Some(first.clone()) {
                        e.prevent_default();
                        last.dyn_ref::<HtmlElement>().unwrap().focus().ok();
                    }
                } else {
                    if active == Some(last.clone()) {
                        e.prevent_default();
                        first.dyn_ref::<HtmlElement>().unwrap().focus().ok();
                    }
                }
            }
        }) as Box<dyn FnMut(_)>);

        doc.add_event_listener_with_callback("keydown", handler.as_ref().unchecked_ref()).ok();

        move || {
            doc.remove_event_listener_with_callback("keydown", handler.as_ref().unchecked_ref()).ok();
        }
    });

    // portal в body
    rsx! {
        Portal {
            div {
                class: "fixed inset-0 z-50 flex items-center justify-center",

                // overlay
                div {
                    class: "
                        absolute inset-0 bg-black/50
                        transition-opacity duration-200
                        {if is_open() { "opacity-100" } else { "opacity-0" }}
                    ",

                    onclick: move |_| {
                        if let Some(cb) = &props.onclose {
                            cb.call(());
                        }
                    }
                }

                // content
                div {
                    class: "
                        relative z-10 w-full max-w-lg
                        bg-background rounded-xl shadow-lg
                        transition-all duration-200
                        {if is_open() {
                            "opacity-100 scale-100"
                        } else {
                            "opacity-0 scale-95"
                        }}
                    ",

                    onclick: |e| e.stop_propagation(),

                    {props.children}
                }
            }
        }
    }
}

#[component]
pub fn PopupHeader(children: Element) -> Element {
    rsx! {
        div { class: "px-4 py-3 border-b border-border/40 font-semibold", {children} }
    }
}

#[component]
pub fn PopupBody(children: Element) -> Element {
    rsx! {
        div { class: "p-4 text-sm", {children} }
    }
}

#[component]
pub fn PopupFooter(children: Element) -> Element {
    rsx! {
        div { class: "px-4 py-3 border-t border-border/40 flex justify-end gap-2", {children} }
    }
}

#[component]
pub fn PopupClose() -> Element {
    let ctx = use_context::<PopupContext>();

    rsx! {
        button {
            class: "absolute top-3 right-3 text-foreground/60 hover:text-foreground",

            onclick: move |_| ctx.open.set(false),

            "✕"
        }
    }
}
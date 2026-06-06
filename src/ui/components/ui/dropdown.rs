use dioxus::{html::g::class, prelude::*};

#[derive(Clone)]
pub struct DropdownContext {
    pub open: Signal<bool>,
    pub position: Signal<Option<(i32, i32)>>,
    pub size: Signal<Option<(i32, i32)>>,
    pub viewport: Signal<Option<(i32, i32)>>,
}

#[component]
pub fn Dropdown(children: Element) -> Element {
    let open = use_signal(|| false);
    let position = use_signal::<Option<(i32, i32)>>(|| None);
    let size = use_signal::<Option<(i32, i32)>>(|| None);
    let viewport = use_signal::<Option<(i32, i32)>>(|| None);

    use_context_provider(|| DropdownContext {
        open,
        position,
        size,
        viewport,
    });

    rsx! {
        div {
            class: "inline-block",
            {children}
        }
    }
}

#[component]
pub fn DropdownTrigger(children: Element) -> Element {
    let mut ctx = use_context::<DropdownContext>();

    let id = use_signal(|| uuid::Uuid::new_v4().to_string());

    rsx! {
        div {
            id: "{id}",
            class: "cursor-pointer",

            onclick: move |_| {
                let mut ctx = ctx.clone();
                let id = id();

                spawn(async move {
                    let data = document::eval(&format!(r#"
                        const rect = document.getElementById('{id}').getBoundingClientRect();

                        return {{
                            x: rect.left,
                            y: rect.top,
                            width: rect.width,
                            height: rect.height,
                            viewportWidth: window.innerWidth,
                            viewportHeight: window.innerHeight
                        }};
                    "#))
                    .await
                    .unwrap();

                    let x = data["x"].as_f64().unwrap_or(0.0) as i32;
                    let y = data["y"].as_f64().unwrap_or(0.0) as i32;
                    let width = data["width"].as_f64().unwrap_or(0.0) as i32;
                    let height = data["height"].as_f64().unwrap_or(0.0) as i32;

                    let viewport_width =
                        data["viewportWidth"].as_f64().unwrap_or(0.0) as i32;

                    let viewport_height =
                        data["viewportHeight"].as_f64().unwrap_or(0.0) as i32;

                    ctx.position.set(Some((x, y)));
                    ctx.size.set(Some((width, height)));
                    ctx.viewport
                        .set(Some((viewport_width, viewport_height)));

                    let current = *ctx.open.read();
                    ctx.open.set(!current);
                });
            },

            {children}
        }
    }
}

#[component]
pub fn DropdownContent(children: Element) -> Element {
    let mut ctx = use_context::<DropdownContext>();

    if !*ctx.open.read() {
        return rsx! {};
    }

    let (trigger_x, trigger_y) =
        ctx.position.read().unwrap_or((0, 0));

    let (_, trigger_height) =
        ctx.size.read().unwrap_or((0, 0));

    let (viewport_width, viewport_height) =
        ctx.viewport.read().unwrap_or((600, 800));

    let dropdown_width = 192;
    let estimated_height = 200;

    let mut left = trigger_x;
    let mut top = trigger_y + trigger_height + 4;

    if left + dropdown_width > viewport_width {
        left = viewport_width - dropdown_width - 16;
    }

    if left < 8 {
        left = 8;
    }

    if top + estimated_height > viewport_height {
        top = trigger_y - estimated_height - 4;
    }

    rsx! {
        div {
            class: "w-full h-full fixed inset-0 z-9998",
            onclick: move |_| ctx.open.set(false)
        }
        div {
            style: format!(
                "
                position: fixed;
                left: {}px;
                top: {}px;
                z-index: 9999;
                ",
                left,
                top
            ),

            class: "
                bg-background
                border border-border/40
                translate-x-1/2
                rounded-md
                shadow-md
                p-1
                                max-h-62
                overflow-y-auto
            ",

            {children}
        }
    }
}

#[derive(Props, PartialEq, Clone)]
pub struct DropdownItemProps {
    children: Element,
    #[props(optional)]
    onclick: Option<EventHandler<MouseEvent>>,
}

#[component]
pub fn DropdownItem(
    props: DropdownItemProps
) -> Element {
    let mut ctx = use_context::<DropdownContext>();

    rsx! {
        div {
            class: format!("
                px-1.5 py-0.5
                hover:bg-foreground/5
                rounded
                cursor-pointer
                flex justify-between items-center
            "),

            onclick: move |e| {
                ctx.open.set(false);

                if let Some(handler) = &props.onclick {
                    handler.call(e);
                }
            },

            {props.children}
        }
    }
}
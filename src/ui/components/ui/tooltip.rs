use dioxus::{desktop::tao::{event_loop::EventLoop, window::Window}, prelude::*};
use serde_json::json;
use uuid::Uuid;
use std::sync::atomic::{AtomicU64, Ordering};
use tokio::time::{sleep, Duration};

const TOOLTIP_HIDE_DELAY: Duration = Duration::from_millis(10);

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum TooltipAlign {
    Left,
    Right,
    Center,
    Top,
    Bottom,
}

#[derive(Props, PartialEq, Clone)]
pub struct TooltipProps {
    pub children: Element,
    #[props(optional)]
    pub target: Option<Element>,
    #[props(default = String::new())]
    pub text: String,
    #[props(default = TooltipAlign::Top)]
    pub align: TooltipAlign,
    #[props(default = String::new())]
    pub class: String,
    #[props(default = false)]
    pub at_cursor: bool,
    #[props(default = false)]
    pub visible: bool,
    #[props(default = 2)]
    pub gap: u64,
}

#[component]
pub fn Tooltip(props: TooltipProps) -> Element {
    let mut visible = use_signal(|| false);
    let mut position = use_signal(|| [0.0, 0.0]);
    let mut tooltip_size = use_signal(|| [300.0, 120.0]);
    let mut window_size = use_signal(|| [800.0, 600.0]);
    let id: Signal<String> = use_signal(|| Uuid::new_v4().to_string());

    let position_class = match props.at_cursor {
        false => match props.align {
            TooltipAlign::Left => format!("right-full top-1/2 -translate-y-1/2 mr-2"),
            TooltipAlign::Right => format!("left-full top-1/2 -translate-y-1/2 ml-2"),
            TooltipAlign::Center => format!("left-1/2 top-full -translate-x-1/2 mt-2"),
            TooltipAlign::Top => format!("left-1/2 bottom-full -translate-x-1/2 mb-2"),
            TooltipAlign::Bottom => format!("left-1/2 top-full -translate-x-1/2 mt-2"),
        },
        true => match props.align {
            TooltipAlign::Left => format!("-translate-y-1/2 -translate-x-full -ml-{}", props.gap),
            TooltipAlign::Right => format!("-translate-y-1/2 translate-x-full -mr-{}", props.gap),
            TooltipAlign::Center => format!("-translate-x-1/2 -translate-y-1/2"),
            TooltipAlign::Top => format!("-translate-x-1/2 -translate-y-full -mt-{}", props.gap),
            TooltipAlign::Bottom => format!("-translate-x-1/2 translate-y-full -mb-{}", props.gap),
        },
    };

    rsx! {
        div {
            class: "relative w-full h-full {props.class}",

            onmouseenter: move |evt| async move  {
                spawn(async move {
                visible.set(true);

                tokio::task::yield_now().await;
                sleep(Duration::from_millis(10)).await;
                if props.at_cursor {
                    evt.stop_propagation();
                    
                    let data = document::eval(
                        format!(
                            r#"
                            const el = document.getElementById("{}");

                            return {{
                                found: el != null,
                                width: el?.offsetWidth ?? 300,
                                height: el?.offsetHeight ?? 120,
                                windowWidth: window.innerWidth,
                                windowHeight: window.innerHeight
                            }};
                            "#,
                            id()
                        )
                        .as_str(),
                    )
                    .await
                    .unwrap_or(json!({
                        "width": 300,
                        "height": 120,
                        "windowWidth": 800,
                        "windowHeight": 600
                    }));

                    let tooltip_width = data["width"]
                        .as_f64()
                        .unwrap_or(300.0);

                    let tooltip_height = data["height"]
                        .as_f64()
                        .unwrap_or(120.0);

                    let window_width = data["windowWidth"]
                        .as_f64()
                        .unwrap_or(800.0);

                    let window_height = data["windowHeight"]
                        .as_f64()
                        .unwrap_or(600.0);

                    tooltip_size.set([tooltip_width, tooltip_height]);
                    window_size.set([window_width, window_height]);

                    const BORDER_GAP: f64 = 4.0;
                    const BORDER_RIGHT_GAP: f64 = 14.0;
                    
                    let evt_x: f64 = evt.client_coordinates().x;
                    let evt_y: f64 = evt.client_coordinates().y;
                    
                    let client_w = window_width;
                    let client_h = window_height;

                    let half_w = tooltip_width / 2.0;
                    let half_h = tooltip_height / 2.0;


                    let x = (evt_x)
                        .max(half_w + BORDER_GAP)
                        .min(client_w - half_w - BORDER_RIGHT_GAP);

                    let y = (evt_y)
                        .max(half_h + BORDER_GAP)
                        .min(client_h - half_h - BORDER_GAP);

                    position.set([x, y]);
                };
                });

            },

            onmouseleave: move |_| {
                spawn(async move {
                    sleep(TOOLTIP_HIDE_DELAY).await;
                    visible.set(false);
                });
            },

            onfocusin: move |_| {
                spawn(async move {
                    sleep(TOOLTIP_HIDE_DELAY).await;
                    visible.set(true);
                });
            },

            onfocusout: move |_| {
                spawn(async move {
                    sleep(TOOLTIP_HIDE_DELAY).await;
                    visible.set(false);
                });
            },

            onmousemove: move |evt: Event<MouseData>| async move {
                if props.at_cursor {
                    evt.stop_propagation();

                    const BORDER_GAP: f64 = 4.0;
                    const BORDER_RIGHT_GAP: f64 = 14.0;
                    
                    let evt_x: f64 = evt.client_coordinates().x;
                    let evt_y: f64 = evt.client_coordinates().y;
                    
                    let client_w = window_size()[0];
                    let client_h = window_size()[1];

                    let half_w = tooltip_size()[0] / 2.0;
                    let half_h = tooltip_size()[1] / 2.0;


                    let x = (evt_x)
                        .max(half_w + BORDER_GAP)
                        .min(client_w - half_w - BORDER_RIGHT_GAP);

                    let y = (evt_y)
                        .max(tooltip_size()[1] + BORDER_GAP * 3.0)
                        .min(client_h - BORDER_GAP);

                    position.set([x, y]);
                };
            },

            {props.children}

            if visible() || props.visible {
                div {
                    role: "tooltip",
                    id: id(),
                    class: "absolute pointer-events-none whitespace-nowrap tooltip rounded-md border border-border/40 bg-secondary/70 px-2 py-1 text-xs text-foreground shadow-sm backdrop-blur-md {position_class}",

                    style: match props.at_cursor {
                        true => {
                            format!(
                                "position: fixed; left: {}px; top: {}px; z-index: 2147483647;",
                                position()[0],
                                position()[1],
                            )
                        }
                        false => "z-index: 2147483647;".to_string(),
                    },

                    {
                        match props.target.clone() {
                            Some(target) => target,
                            None => rsx! { "{props.text}" },
                        }
                    }
                }
            }
        }
    }
}
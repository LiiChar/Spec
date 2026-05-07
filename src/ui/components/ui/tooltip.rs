use dioxus::prelude::*;
use std::sync::atomic::{AtomicU64, Ordering};
use tokio::time::{sleep, Duration};

const TOOLTIP_HIDE_DELAY: Duration = Duration::from_millis(10);

#[derive(Debug, PartialEq, Clone)]
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
            TooltipAlign::Bottom => format!("-translate-x-1/2  translate-y-full -mb-{}", props.gap),
        },
    };

    rsx! {
        div {
            class: "relative w-full h-full {props.class}",
            onmouseenter: move |_| {
                spawn(async move {
                        sleep(TOOLTIP_HIDE_DELAY).await;
                        visible.set(true);
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
            onmousemove: move |evt| {
                if props.at_cursor {
                    evt.stop_propagation();
                    let x: f64 = evt.client_coordinates().x;
                    let y: f64 = evt.client_coordinates().y;
                    position.set([x, y]);
                }
            },

            {props.children}

            if visible() || props.visible {
                div {
                    role: "tooltip",
                    class: "absolute pointer-events-none whitespace-nowrap rounded-md border backdrop-blur-md border-border/40 bg-secondary/50 px-2 py-1 text-xs text-foreground shadow-sm backdrop-blur-md {position_class} ",
                    style: match props.at_cursor {
                        true => format!("position: fixed; left: {}px; top: {}px; z-index: 2147483647;", position()[0], position()[1]),
                        false => "z-index: 2147483647;".to_string(),
                    },
                    {
                        match props.target.clone() {
                            Some(target) => target,
                            None => {
                                rsx! {
                                    "{props.text}"
                                }
                            },
                        }
                    }
                }
            }
        }

    }
}

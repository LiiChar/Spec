use dioxus::{desktop::tao::{event_loop::EventLoop, window::Window}, prelude::*};
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

/* =========================
   helpers (NEW LOGIC)
========================= */

#[derive(Clone, Copy)]
struct Rect {
    x: f64,
    y: f64,
    w: f64,
    h: f64,
}

fn viewport_size() -> (f64, f64) {
    let mut event_loop = EventLoop::new();
    let window = Window::new(&event_loop).unwrap();

    let size = window.inner_size();

    (size.width as f64, size.height as f64)
}

fn apply_offset(x: f64, y: f64, align: TooltipAlign, gap: f64) -> (f64, f64) {
    match align {
        TooltipAlign::Top => (x, y - gap),
        TooltipAlign::Bottom => (x, y + gap),
        TooltipAlign::Left => (x - gap, y),
        TooltipAlign::Right => (x + gap, y),
        TooltipAlign::Center => (x, y),
    }
}

fn flip(
    mut x: f64,
    mut y: f64,
    align: TooltipAlign,
    tooltip: Rect,
    vw: f64,
    vh: f64,
    gap: f64,
) -> (f64, f64, TooltipAlign) {
    let mut new_align = align;

    let fits_right = x + tooltip.w < vw;
    let fits_left = x - tooltip.w > 0.0;
    let fits_top = y - tooltip.h > 0.0;
    let fits_bottom = y + tooltip.h < vh;

    match align {
        TooltipAlign::Right if !fits_right && fits_left => {
            new_align = TooltipAlign::Left;
            x -= tooltip.w + gap;
        }
        TooltipAlign::Left if !fits_left && fits_right => {
            new_align = TooltipAlign::Right;
            x += gap;
        }
        TooltipAlign::Top if !fits_top && fits_bottom => {
            new_align = TooltipAlign::Bottom;
            y += gap;
        }
        TooltipAlign::Bottom if !fits_bottom && fits_top => {
            new_align = TooltipAlign::Top;
            y -= tooltip.h + gap;
        }
        _ => {}
    }

    (x, y, new_align)
}

fn shift(mut x: f64, mut y: f64, tooltip: Rect, vw: f64, vh: f64) -> (f64, f64) {
    if x + tooltip.w > vw {
        x = vw - tooltip.w - 8.0;
    }
    if x < 0.0 {
        x = 8.0;
    }

    if y + tooltip.h > vh {
        y = vh - tooltip.h - 8.0;
    }
    if y < 0.0 {
        y = 8.0;
    }

    (x, y)
}

fn compute_position(
    x: f64,
    y: f64,
    align: TooltipAlign,
    gap: f64,
) -> (f64, f64, TooltipAlign) {
    let (vw, vh) = viewport_size();

    // примерные размеры tooltip (можно улучшить через измерение DOM)
    let tooltip = Rect {
        x: 0.0,
        y: 0.0,
        w: 180.0,
        h: 32.0,
    };

    let (mut x, mut y) = apply_offset(x, y, align, gap as f64);
    let (x2, y2, align2) = flip(x, y, align, tooltip, vw, vh, gap as f64);
    let (x3, y3) = shift(x2, y2, tooltip, vw, vh);

    (x3, y3, align2)
}

/* =========================
   COMPONENT
========================= */

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
            TooltipAlign::Bottom => format!("-translate-x-1/2 translate-y-full -mb-{}", props.gap),
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

                    // let (nx, ny, _) = compute_position(
                    //     x,
                    //     y,
                    //     props.align,
                    //     props.gap as f64,
                    // );

                    position.set([x, y]);
                }
            },

            {props.children}

            if visible() || props.visible {
                div {
                    role: "tooltip",
                    class: "absolute pointer-events-none whitespace-nowrap rounded-md border backdrop-blur-md border-border/40 bg-secondary/50 px-2 py-1 text-xs text-foreground shadow-sm backdrop-blur-md {position_class}",

                    style: match props.at_cursor {
                        true => format!(
                            "position: fixed; left: {}px; top: {}px; z-index: 2147483647;",
                            position()[0],
                            position()[1]
                        ),
                        false => "z-index: 2147483647;".to_string(),
                    },

                    {
                        match props.target.clone() {
                            Some(target) => target,
                            None => rsx! {
                                "{props.text}"
                            },
                        }
                    }
                }
            }
        }
    }
}
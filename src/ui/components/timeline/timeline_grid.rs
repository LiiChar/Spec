use chrono::{Local, Timelike};
use dioxus::prelude::*;

use dioxus_free_icons::icons::ld_icons::{LdArrowUpToLine, LdBarChart, LdX, LdArrowDownToLine };
use dioxus_free_icons::Icon;

use crate::{lib::Segment, ui::components::timeline::timeline_select::TimelineSelect};

#[derive(Props, PartialEq, Clone)]
pub struct TimelineGridProps {
    pub start_hour: u32,
    pub end_hour: u32,
    pub selected_hour: Signal<Option<u32>>,
}

#[component]
pub fn TimelineGrid(mut props:  TimelineGridProps) -> Element {
    let start_hour = props.start_hour;
    
    let end_hour = props.end_hour;
    
    let hour_count = (end_hour - start_hour + 1).max(1);

    let is_selected = (props.selected_hour)()
        .map(|h| h >= start_hour && h <= end_hour)
        .unwrap_or(false) && hour_count == 1;
    
    let is_current_hour = {
        let now = Local::now().hour();
        now >= start_hour && now <= end_hour
    };

    rsx! {
        if hour_count > 1 {
            div { class: "absolute z-4 left-2 top-1 text-xs opacity-60 pointer-events-none select-none", "{start_hour}:00-{end_hour}:00"
          }
        } else {
            div { 
                onclick: move |evt| {
                    evt.stop_propagation();
                    let current_hour = (props.selected_hour)();
                    if current_hour.is_some() {
                        if current_hour.unwrap() == start_hour {
                            props.selected_hour.set(None);
                        } else {
                            props.selected_hour.set(Some(start_hour));
                        }
                    } else {
                        props.selected_hour.set(Some(start_hour));
                    }
                },
                class: format!("absolute z-5 left-{} top-1 text-xs opacity-60 select-none flex gap-1 cursor-pointer",                                         if is_selected { "2" } else { "1" }), 
                "{start_hour}:00 ",
                if is_selected {
                    button {
                        Icon { icon: LdArrowUpToLine, height: 10, width: 10 }
                    }
                } else {
                    button {
                        Icon { icon: LdArrowDownToLine, height: 10, width: 10 }
                    }
                }
            }
        }

        div {
            class: "flex justify-evenly absolute top-0 left-0 h-full w-full z-0",
            {(0..5).map(|i| {
                rsx! {
                    div { class: "h-full w-[1px] border-dashed border-border/10 border-l-[1px] " }
                }
            })}
        }
        if is_selected {
            {(1..4).map(|i| {
                rsx! {
                    div { class: "absolute left-2 z-40 text-xs opacity-60 pointer-events-none select-none z-1 -translate-y-1/2", style: format!("top: calc(100%/4*{})", i), {format!("{}:{}",start_hour, 60 / 4 * i)} }
                }
            })}
        }

        if is_selected {
            {(1..12).map(|i| {
                let minute = i * 5;

                let width = if minute % 15 == 0 {
                    4
                } else if minute % 10 == 0 {
                    3
                } else {
                    2
                };

                rsx! {
                    div {
                        class: "absolute left-1 z-40 h-[1px] bg-border",
                        style: format!(
                            "top: calc(100% / 12 * {}); width: {}px; left: {}px",
                            i,
                            width,
                            if is_current_hour { 3 } else { 0 }
                        ),
                    }
                }
            })}
        } else if start_hour != end_hour {
            {(0..end_hour-start_hour+1).map(|i| {
                let step = end_hour - start_hour + 1;
                let minute = i * step; 
                let width = 2;
                rsx! {
                    div {
                        class: "absolute left-0 z-40 h-[1px] bg-border",
                        style: format!(
                            "top: calc(100% / {} * {}); width: {}px; left: {}px",
                            step,
                            i,
                            2,
                            if is_current_hour { 4 } else { 0 }
                        ),
                    }
                }
            })}
        } else {
            {(1..4).map(|i| {
                let minute = i * 15; 
                let width = if minute % 30 == 0 {
                    4
                } else {
                    2
                };
                rsx! {
                    div {
                        class: "absolute left-0 z-40 h-[1px] bg-border",
                        style: format!(
                            "top: calc(100% / 4 * {}); width: {}px; left: {}px",
                            i,
                            width,
                            if is_current_hour { 4 } else { 0 }
                        ),
                    }
                }
            })}
        }
    }
  }
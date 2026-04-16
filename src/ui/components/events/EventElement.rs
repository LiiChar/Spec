use std::collections::HashMap;

use chrono::{DateTime, Datelike, Local, NaiveDate, Timelike, Utc};
use dioxus::prelude::*;

use crate::{core::EventModel, lib::convert_ts_to_local_date, ui::TimelineOrientation};

#[derive(Props, PartialEq, Clone)]
pub struct EventsElementProps {
    pub events: Vec<EventModel>,
    #[props(default = 0)]
    pub hour: u32,
    #[props(default = TimelineOrientation::Vertical)]
    pub orientation: TimelineOrientation,
    #[props(default = "".to_string())]
    pub class: String,
    #[props(default = "".to_string())]
    pub style: String, 
    #[props(default = None)]
    pub onclick: Option<EventHandler<MouseEvent>>,
}

#[component]
pub fn EventsElement(props: EventsElementProps) -> Element {
    let events = props.events;

    rsx! {
      div {
          onclick: move |evt| {
              if let Some(onclick) = props.onclick {
                  onclick.call(evt);
              }
          },
          class: format!(
              " rounded-sm w-full h-full relative {} {} {} {}",
                if props.orientation == TimelineOrientation::Horizontal {
                  "max-w-[calc(100%/24)] max-h-full"
              } else {
                  "max-w-full max-h-[calc(100%/24)]"
              },
              if !events.is_empty() {
                  "bg-blue-500/30"
              } else {
                  "bg-zinc-800/30"
              },
              if props.hour == Local::now().hour() {
                  "border border-blue-500 active max-h-[calc(100%/24 + 10px)]"
              } else {
                  "max-h-[calc(100%/24 - 10px)]"
              },
              props.class
          ),
          style: props.style,
          span {
              class: "absolute left-1 text-[10px] opacity-50",
              "{props.hour}"
          }

          {events.iter().map(|e| {
              let start_dt = convert_ts_to_local_date(e.timestamp);
              let end_dt = convert_ts_to_local_date(e.timestamp + e.duration);

              // секунды внутри часа
              let start_sec = start_dt.minute() * 60 + start_dt.second();
              let end_sec = end_dt.minute() * 60 + end_dt.second();

              // 🔥 правильная математика
              let top = (start_sec as f32 / 3600.0) * 100.0;
              let height = ((end_sec.saturating_sub(start_sec)) as f32 / 3600.0) * 100.0;

              rsx! {
                  div {
                      class: "absolute group left-0 bg-secondary/70 rounded-sm",
                      style: match props.orientation {
                          TimelineOrientation::Vertical => format!(
                              "top: {}%; height: {}%; width: 100%;",
                              top,
                              height.max(2.0) // чтобы не исчезало
                          ),
                          TimelineOrientation::Horizontal => format!(
                              "left: {}%; width: {}%; height: 100%;",
                              top,
                              height.max(2.0) // чтобы не исчезало
                          ),
                      },
                      title: format!(
                          "{} - {}",
                          start_dt.time(),
                          end_dt.time()
                      ),
                      span {
                        class: "group-hover:block hidden ",
                        {e.window.title.clone()}
                      }
                  }
              }
          })}
      }
  }
} 
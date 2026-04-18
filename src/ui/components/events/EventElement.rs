use std::collections::HashMap;

use chrono::{DateTime, Datelike, Local, NaiveDate, Timelike, Utc};
use dioxus::prelude::*;

use crate::{core::EventModel, lib::{convert_ts_to_local_date, get_process_color}, ui::TimelineOrientation};

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
pub fn EventElement(props: EventsElementProps) -> Element {
    let events = props.events;

    rsx! {
      div {
          onclick: move |evt| {
              if let Some(onclick) = props.onclick {
                  onclick.call(evt);
              }
          },
          class: format!(
              "rounded-sm w-full h-full relative transition-all duration-200 {} {} {} {}",
                if props.orientation == TimelineOrientation::Horizontal {
                  "max-w-[calc(100%/24)] max-h-full"
              } else {
                  "max-w-full"
              },
              if !events.is_empty() {
                  "bg-zinc-900/50 hover:bg-zinc-800/70"
              } else {
                  "bg-zinc-900/20 hover:bg-zinc-800/30"
              },
              if props.hour == Local::now().hour() {
                  "border-2 border-cyan-400 shadow-lg shadow-cyan-400/50 current-hour min-h-[400px]"
              } else {
                  "border border-zinc-700/50"
              },
              props.class
          ),
          style: {
              match props.orientation {
                  TimelineOrientation::Vertical => format!("height: calc(100%/24);{}", props.style),
                  TimelineOrientation::Horizontal => props.style.clone(),
              }
          },
          
          // Час на боку
          span {
              class: "absolute left-1.5 top-1 text-[10px] font-semibold opacity-60 pointer-events-none",
              {
                  let h = format!("{:02}:00", props.hour);
                  h
              }
          }

          // События
          {events.iter().map(|e| {
              let start_dt = convert_ts_to_local_date(e.timestamp);
              let end_dt = convert_ts_to_local_date(e.timestamp + e.duration);

              // секунды внутри часа
              let start_sec = start_dt.minute() * 60 + start_dt.second();
              let end_sec = end_dt.minute() * 60 + end_dt.second();

              let top = (start_sec as f32 / 3600.0) * 100.0;
              let height = ((end_sec.saturating_sub(start_sec)) as f32 / 3600.0) * 100.0;
              let min_height = 0.01; // Минимальная высота для видимости

              let color = get_process_color(&e.window.process_name);
              let duration_seconds = end_dt.signed_duration_since(start_dt).num_seconds();
              let duration_minutes = duration_seconds as f32 / 60.0;
              
              // Подготавливаем данные для drag & drop
              let process_name = e.window.process_name.clone();
              let pid_str = format!("{}", e.window.pid);

              rsx! {
                  div {
                      class: format!(
                          "absolute group left-0 right-0 {} rounded-sm cursor-pointer transition-all hover:opacity-100 opacity-80",
                          color
                      ),
                      style: match props.orientation {
                          TimelineOrientation::Vertical => format!(
                              "top: {}%; height: {}%; width: 100%;",
                              top,
                              height.max(min_height)
                          ),
                          TimelineOrientation::Horizontal => format!(
                              "left: {}%; width: {}%; height: 100%;",
                              top,
                              height.max(min_height)
                          ),
                      },
                      draggable: "true",
                      ondragstart: move |evt| {
                          let data = evt.data_transfer();
                          let _ = data.set_data("text/plain", &process_name);
                          let _ = data.set_data("application/x-process-info", &pid_str);
                      },
                      
                      // Подробный тултип
                      div {
                          class: "absolute bottom-full left-0 mb-2 hidden group-hover:block z-50 bg-zinc-950 text-white p-2 rounded-md shadow-lg text-xs whitespace-nowrap border border-zinc-700",
                          style: "min-width: 200px;",
                          
                          div { class: "font-bold text-cyan-400", "{e.window.process_name}" }
                          div { class: "text-gray-300 truncate", "{e.window.title}" }
                          div { class: "text-gray-400 text-[10px]", "{start_dt.time()} → {end_dt.time()}" }
                          div { class: "text-amber-300", "{duration_minutes} м" }
                      }
                      
                      // Текст на полоске (если достаточно места)
                      if height > 8.0 {
                          span {
                              class: "absolute inset-0 flex items-center justify-center text-[9px] font-bold text-white opacity-70 truncate pointer-events-none",
                              "{duration_minutes}м"
                          }
                      }
                  }
              }
          })}
      }
  }
} 
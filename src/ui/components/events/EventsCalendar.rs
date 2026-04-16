use std::collections::HashMap;

use chrono::{Datelike, Local, NaiveDate};
use dioxus::prelude::*;

use crate::core::EventModel;

#[derive(Props, PartialEq, Clone)]
pub struct EventsCalendarProps {
    events: Vec<EventModel>
}

fn is_today(date: NaiveDate) -> bool {
    date == Local::now().date_naive()
}

fn group_by_day<T>(
    events: Vec<(NaiveDate, T)>
) -> HashMap<NaiveDate, Vec<T>> {
    let mut map = HashMap::new();

    for (date, event) in events {
        map.entry(date).or_insert(vec![]).push(event);
    }

    map
}

pub fn build_month_matrix(date: NaiveDate) -> Vec<Vec<NaiveDate>> {
    let first_day = date.with_day(1).unwrap();

    // индекс дня недели (0 = Mon ... 6 = Sun)
    let start_offset = first_day.weekday().num_days_from_monday();

    let mut current = first_day - chrono::Duration::days(start_offset as i64);

    let mut weeks: Vec<Vec<NaiveDate>> = Vec::new();

    // обычно календарь = 5–6 недель
    for _ in 0..6 {
        let mut week = Vec::new();

        for _ in 0..7 {
            week.push(current);
            current = current.succ_opt().unwrap();
        }

        weeks.push(week);
    }

    weeks
}


#[component]
pub fn EventsCalendar(props: EventsCalendarProps) -> Element {
    let today = Local::now().date_naive();

    let calendar = build_month_matrix(today);

    rsx! {
      div {
        class: "flex flex-col gap-1 w-full",
        {calendar.iter().map(|week| rsx! { 
          div {
            class: "flex flex-row gap-1",
            {week.iter().map(|date| rsx! { 
              div { 
                class: "w-[calc(100%/7)] h-[calc(100%/7)] flex items-center justify-center rounded-md",
                "{date.day()}"
              }
            })}
          }
        })}
      }
    }
} 
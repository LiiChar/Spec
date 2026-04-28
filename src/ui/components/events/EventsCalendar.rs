use std::collections::HashMap;

use chrono::{Datelike, Local, NaiveDate};
use dioxus::prelude::*;

use crate::core::EventModel;

#[derive(Props, PartialEq, Clone)]
pub struct EventsCalendarProps {
    #[props(default = vec![])]
    events: Vec<EventModel>,
    #[props(default = Local::now().date_naive())]
    day: NaiveDate,
    #[props(!optional)]
    onselect: EventHandler<NaiveDate>,
}

fn is_today(date: NaiveDate, today: NaiveDate) -> bool {
    date == today
}

fn in_month(date: NaiveDate) -> bool {
    date.month() == Local::now().date_naive().month() && date.year() == Local::now().date_naive().year()
}

pub fn build_month_matrix(date: NaiveDate) -> Vec<Vec<NaiveDate>> {
  let first_day = date.with_day(1).unwrap();

  // индекс дня недели (0 = Mon ... 6 = Sun)
  let start_offset = first_day.weekday().num_days_from_monday();

  let mut current = first_day - chrono::Duration::days(start_offset as i64);

  let total_days = start_offset as usize + date.num_days_in_month() as usize;
  let count_week = (total_days + 6) / 7;

  let mut weeks: Vec<Vec<NaiveDate>> = Vec::new();

  // обычно календарь = 5–6 недель
  for _ in 0..count_week {
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
    let today = props.day;

    let calendar = build_month_matrix(today);

    let day_week = vec!["Пн", "Вт", "Ср", "Чт", "Пт", "Сб", "Вс"];

    rsx! {
      div {
        class: "flex flex-col gap-1 w-full bg-background p-1 border border-border/30 rounded-lg",
        div {
          class: "flex flex-row gap-1 ",
          {day_week.iter().cloned().map(|day| {
              rsx! {
                  div {
                      class: "text-center text-xs font-semibold text-muted-foreground/70 uppercase tracking-wide p-1 w-[calc(100%/7)]",
                      "{day}"
                  }
              }
          })}
        }
        {calendar.iter().cloned().map(|week| rsx! {
            div {
                class: "flex flex-row gap-1 ",

                {week.into_iter().map(|date| {
                    let date_copy = date;

                    rsx! {
                        div {
                            onclick: move |_| props.onselect.call(date_copy),
                            class: format!(
                                "w-[calc(100%/7)] h-[calc(100%/7)] cursor-pointer flex items-center hover:bg-foreground/80 transition-all hover:text-background justify-center rounded-sm aspect-square p-1 {} {} {}",
                                if !in_month(date_copy) { "text-muted-foreground/40" } else { "" },
                                if is_today(date_copy, today) { "bg-foreground text-background! shadow-md" } else { "" },
                                if Local::now().date_naive() == date_copy { "border-[1px] border-foreground" } else { "" },
                            ),
                            "{date_copy.day()}"
                        }
                    }
                })}
            }
        })}
      }
    }
} 
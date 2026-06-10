use chrono::{Datelike, Local, NaiveDate, TimeZone};
use dioxus::prelude::*;

use dioxus_free_icons::{icons::ld_icons::LdCalendar, Icon};

use crate::ui::{button::{Button, ButtonSize, ButtonVariant}, input::Input};

#[derive(Props, PartialEq, Clone)]
pub struct CalendarProps {
    #[props(default = Local::now().date_naive())]
    default_date: NaiveDate,
    #[props(!optional)]
    onselect: EventHandler<NaiveDate>,
}

fn is_today(date: NaiveDate, today: NaiveDate) -> bool {
    date == today
}

fn in_month(date: NaiveDate) -> bool {
    date.month() == Local::now().date_naive().month()
        && date.year() == Local::now().date_naive().year()
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
pub fn Calendar(props: CalendarProps) -> Element {
    let today = Local::now().date_naive();

    let mut current_day = use_signal(|| props.default_date);

    let calendar = build_month_matrix(current_day.read().clone());

    let day_week = vec!["Пн", "Вт", "Ср", "Чт", "Пт", "Сб", "Вс"];

    let mut visible = use_signal(|| false);

    rsx! {
        div { class: "relative",
            div {
                Input {
                    r#type: "date",
                    class: "text-sm",
                    oninput: move |evt: FormEvent| {
                        let value = evt.value();
                        println!("Value: {}", value);
                        if let Ok(date) = NaiveDate::parse_from_str(&value, "%Y-%m-%d") {
                            current_day.set(date);
                            props.onselect.call(date)
                        }
                    },
                    value: format!("{}", current_day.read().format("%Y-%m-%d")),
                    placeholder: "дд.мм.гггг",
                }
                div {
                    onclick: move |_| visible.set(!visible()),
                    class: "absolute right-3 top-1/2 -translate-y-1/2 text-xl group",
                    Icon {
                        icon: LdCalendar,
                        width: 18,
                        height: 18,
                        class: "stroke-foreground/30 group-hover:stroke-foreground",
                    }
                }
            }
            if visible() {
                div { class: "absolute min-w-[200px] -bottom-1 translate-y-full flex flex-col gap-1 w-full bg-background p-1 border border-border/30 rounded-lg",
                    Button {
                        onclick: move |evt: MouseEvent| {
                            evt.stop_propagation();
                            let naive_next_month = current_day
                                .read()
                                .checked_sub_months(chrono::Months::new(1))
                                .expect("Failed sub month to current date");
                            let cl_day = Local
                                .from_local_datetime(&naive_next_month.and_hms_opt(0, 0, 0).unwrap())
                                .unwrap()
                                .date_naive();
                            current_day.set(cl_day);
                            props.onselect.call(cl_day)
                        },
                        class: "absolute left-0 -bottom-1 min-h-[26px] min-w-[26px] h-[26px] w-[26px] translate-y-full flex items-center justify-center rounded hover:bg-primary/10 transition-colors focus:ring-1 focus:ring-primary",
                        aria_label: "Предыдущий месяц",
                        "←"
                    }
                    Button {
                        onclick: move |evt: MouseEvent| {
                            evt.stop_propagation();
                            let naive_next_month = current_day
                                .read()
                                .checked_add_months(chrono::Months::new(1))
                                .expect("Failed add month to current date");
                            let cl_day = Local
                                .from_local_datetime(&naive_next_month.and_hms_opt(0, 0, 0).unwrap())
                                .unwrap()
                                .date_naive();
                            current_day.set(cl_day);
                            props.onselect.call(cl_day)
                        },
                        class: "absolute right-0 -bottom-1 min-h-[26px] min-w-[26px] h-[26px] w-[26px] translate-y-full flex items-center justify-center rounded hover:bg-primary/10 transition-colors focus:ring-1 focus:ring-primary",
                        aria_label: "Следующий месяц",
                        "→"
                    }
                    div { class: "flex flex-row gap-1 ",
                        {
                            day_week
                                .iter()
                                .cloned()
                                .map(|day| {
                                    rsx! {
                                        div { class: "text-center text-xs font-semibold text-muted-foreground/70 uppercase tracking-wide p-1 w-[calc(100%/7)]",
                                            "{day}"
                                        }
                                    }
                                })
                        }
                    }
                    {calendar.iter().cloned().map(|week| rsx! {
                        div { class: "flex flex-row gap-1 ",

                            {
                                week.into_iter()
                                    .map(|date| {
                                        let date_copy = date;
                                        rsx! {
                                            div {
                                                onclick: move |_| {
                                                    current_day.set(date_copy);
                                                    props.onselect.call(date_copy);
                                                    visible.set(false);
                                                },
                                                class: format!(
                                                    "w-[calc(100%/7)] h-[calc(100%/7)] cursor-pointer flex items-center hover:bg-foreground/80 transition-all hover:text-background justify-center rounded-sm aspect-square p-1 {} {} {}",
                                                    if !in_month(date_copy) { "text-muted-foreground/40" } else { "" },
                                                    if is_today(date_copy, current_day.read().clone()) {
                                                        "bg-foreground text-background! shadow-md"
                                                    } else {
                                                        ""
                                                    },
                                                    if today == date_copy { "border-[1px] border-foreground" } else { "" },
                                                ),
                                                tabindex: 0,
                                                role: "button",
                                                aria_label: format!(
                                                    "Дата: {}, {}",
                                                    date_copy.format("%d %B %Y"),
                                                    if is_today(date_copy, current_day.read().clone()) {
                                                        "сегодня"
                                                    } else {
                                                        ""
                                                    },
                                                ),
                                                "{date_copy.day()}"
                                            }
                                        }
                                    })
                            }
                        }
                    })}
                }
            }
        
        }
    }

}

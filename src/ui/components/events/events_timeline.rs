use std::collections::HashMap;

use chrono::{DateTime, Datelike, Duration, Local, Timelike};
use dioxus::prelude::*;
use dioxus_free_icons::icons::ld_icons::{LdArrowUpToLine, LdBarChart, LdX, LdArrowDownToLine };
use dioxus_free_icons::Icon;


use crate::lib::{group_by_segments, merge_visual_density, y_to_timestamp};
use crate::ui::components::events::EventElement;
use crate::ui::components::modal::job_modal::JobModal;
use crate::ui::components::timeline::timeline_grid::TimelineGrid;
use crate::ui::components::timeline::timeline_select::TimelineSelect;
use crate::ui::components::timeline::timeline_time::TimelineTime;
use crate::ui::context::use_settings;
use crate::{
    core::{EventModel, EventType, JobModel},
    lib::{CronExpr, convert_ts_to_local_date, merge_events},
};

#[derive(PartialEq, Clone, Copy)]
pub enum TimelineOrientation {
    Horizontal,
    Vertical,
}

#[derive(Props, PartialEq, Clone)]
pub struct EventsCalendarProps {
    events: ReadSignal<Vec<EventModel>>,
    jobs: ReadSignal<Vec<JobModel>>,
    day: ReadSignal<DateTime<Local>>,
    orientation: TimelineOrientation,
}


#[component]
pub fn EventsTimeline(props: EventsCalendarProps) -> Element {
    let settings = use_settings();

    let mut selected_hour = use_signal(|| None::<u32>);
    let mut selected_job = use_signal(|| None::<JobModel>);

    let day_events = use_memo(move || {
        let show_idle = settings.settings.read().show_idle_events;

        let merged = {
            let items = props.events.read();

            let items = items
                .iter()
                .filter_map(|i| {
                    if !show_idle && i.event_type == EventType::Idle {
                        None
                    } else {
                        Some(i.clone())
                    }
                })
                .collect::<Vec<_>>();

            let merged = merge_events(items);

            let px_per_hour = selected_hour()
                .map(|_| 800.0)
                .unwrap_or(80.0);

            let merged = merge_visual_density(merged, px_per_hour, 3.0);
            merged
        };

        let segments = group_by_segments(&merged);
        let today = props.day.read();

        segments
            .into_iter()
            .filter(|segment| {
                today.format("%h%d").to_string() == segment.day.format("%h%d").to_string()
            })
            .collect::<Vec<_>>()
    });

    rsx! {

        JobModal {
            job: selected_job.read().clone(),
            on_close: move |_| selected_job.set(None),
        }
        div {
            class: format!(
                "flex w-full h-min relative rounded-md overflow-hidden border border-border/50 bg-card/30 user-select-none {}",
                match props.orientation {
                    TimelineOrientation::Horizontal => "flex-row",
                    TimelineOrientation::Vertical => "flex-col",
                },
            ),
            TimelineSelect {
                segments: (day_events.clone())(),
                selected_hour: selected_hour,
            }
            TimelineTime {
                segments: (day_events.clone())(),
                selected_hour: selected_hour,
            }
            {
                day_events
                    .iter()
                    .map(|segment| {
                        let start_hour = segment.start;
                        let end_hour = segment.end;
                        let hour_count = (end_hour - start_hour + 1).max(1);
                        let is_selected = selected_hour()
                            .map(|h| h >= start_hour && h <= end_hour)
                            .unwrap_or(false) && hour_count == 1;
                        let height = if is_selected { settings.settings.read().selected_segment_height as f32 } else { settings.settings.read().segment_height as f32 };
                        let start_ts = props
                            .day
                            .read()
                            .date_naive()
                            .and_hms_opt(0, 0, 0)
                            .unwrap()
                            .and_local_timezone(Local)
                            .unwrap()
                            .timestamp_millis();
                        let jobs = props.jobs.read();
                        let mut event_jobs: Vec<JobModel> = Vec::new();

                        let segment_st_timestamp =
                            start_ts + start_hour as i64 * 60 * 60 * 1000;

                        let segment_ed_timestamp =
                            start_ts + (end_hour as i64 + 1) * 60 * 60 * 1000;

                        jobs.iter().for_each(|job| {
                            let (st_timestamp, ed_timestamp) = if job.start_ts > 86_400_000 {
                                (
                                    job.start_ts,
                                    job.end_ts,
                                )
                            } else {
                                (
                                    start_ts + job.start_ts,
                                    start_ts + job.end_ts,
                                )
                            };

                            let c = job.cron.clone().unwrap_or("* * * * * * *".to_string());
                            let cron = CronExpr::parse(c.as_str()).unwrap_or_else(|_| CronExpr::parse("* * * * * * *").unwrap());

                            let dt = Local::now();

                            let is_today = cron.matches(dt, Some(String::from("- - - - + - -")));
                            
                            if (st_timestamp <= segment_ed_timestamp
                                && ed_timestamp >= segment_st_timestamp) && is_today
                            {
                                event_jobs.push(job.clone());
                            }

                        });

                        rsx! {
                            div {
                                key: "{start_hour}-{end_hour}", 
                                class: format!("relative border-dashed border-border/10 border-b-[1px] last:border-b-0"),

                                style: format!("height: {}px;", height as i32),

                                EventElement {
                                    key: "{start_hour}-{end_hour}",
                                    class: "h-full border-none!",
                                    events: segment.group.clone(),
                                    jobs: event_jobs.clone(),
                                    selected_job,
                                    start_hour,
                                    end_hour,
                                    orientation: props.orientation.clone(),
                                    style: format!("height: {}px;", height as i32),
                                    is_selected: is_selected.clone(),
                                }

                                TimelineGrid {
                                    end_hour: end_hour,
                                    start_hour: start_hour,
                                    selected_hour: selected_hour,
                                }
                            }
                        }
                    })
            }
        }
    }
}

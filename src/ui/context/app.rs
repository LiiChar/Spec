use dioxus::prelude::*;

use chrono::{DateTime, Local};

use crate::{
    core::{EventModel, JobModel},
    ui::Page,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AppProvider {
    pub page: Signal<Page>,
    pub day: Signal<DateTime<Local>>,
    pub time: Signal<i64>,
    pub start_time: Signal<Option<i64>>,
    pub events: Signal<Vec<EventModel>>,
    pub jobs: Signal<Vec<JobModel>>,
}

impl Default for AppProvider {
    fn default() -> Self {
        let now = Local::now().timestamp_millis();

        Self {
            page: Signal::new(Page::Main),
            events: Signal::new(Vec::new()),
            day: Signal::new(Local::now()),
            time: Signal::new(now as i64),
            start_time: Signal::new(None),
            jobs: Signal::new(Vec::new()),
        }
    }
}

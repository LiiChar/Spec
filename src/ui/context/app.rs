use dioxus::prelude::*;

use chrono::{DateTime, Local};

use crate::{
    core::{EventModel, GoalModel, JobModel, with_database, with_database_mut},
    ui::Page,
};

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum AppVariant {
    #[default]
    Events,
    Tags,
    Jobs,
}

impl AppVariant {
    pub fn as_str(&self) -> &'static str {
        match self {
            AppVariant::Events => "events",
            AppVariant::Tags => "tags",
            AppVariant::Jobs => "jobs",
        }
    }
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "events" => Some(AppVariant::Events),
            "tags" => Some(AppVariant::Tags),
            "jobs" => Some(AppVariant::Jobs),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AppProvider {
    pub page: Signal<Page>,
    pub day: Signal<DateTime<Local>>,
    pub time: Signal<i64>,
    pub start_time: Signal<Option<i64>>,
    pub events: Signal<Vec<EventModel>>,
    pub jobs: Signal<Vec<JobModel>>,
    pub goals: Signal<Vec<GoalModel>>,
    pub variant: Signal<AppVariant>,
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
            goals: Signal::new(Vec::new()),
            variant: Signal::new(AppVariant::Events),
        }
    }
}


impl AppProvider {
    pub fn refresh_events(&self) {
        let mut ctx_events: Signal<Vec<EventModel>> = self.events;
        spawn(async move {
            let result = tokio::task::spawn_blocking(move || {
                with_database(|db| {
                    db.get_all_events().unwrap_or(Vec::new())
                })
            })
            .await;

            match result {
                Ok(events) => {
                    ctx_events.set(events);
                }
                Err(e) => println!("Task error: {:?}", e),
            }
        });
    }
    pub fn update_jobs(&self, job: JobModel) {
        let mut ctx_job: Signal<Vec<JobModel>> = self.jobs;
        let name = job.name.clone();
        spawn(async move {
            let result = tokio::task::spawn_blocking(move || {
                with_database_mut(|db| {
                    if job.id.is_some() {
                        println!("Update job with name: {}", name);
                        db.update_job(&job)
                    } else {
                        println!("Saved job with name: {}", name);
                        db.save_job(&job).map(|_| ())
                    }
                })
            })
            .await;

            match result {
                Ok(Ok(id)) => {
                    if let Ok(jobs) = with_database(|db| db.get_jobs()) {
                       ctx_job.set(jobs);
                    }
                }
                Ok(Err(e)) => println!("DB error: {:?}", e),
                Err(e) => println!("Task error: {:?}", e),
            }
        });
    }
    pub fn delete_job(&self, id: i64) {
        let mut ctx_job: Signal<Vec<JobModel>> = self.jobs;
        spawn(async move {
            let result = tokio::task::spawn_blocking(move || {
                with_database_mut(|db| {
                    db.delete_job(id).map(|_| ())
                })
            })
            .await;

            match result {
                Ok(Ok(_)) => {
                    if let Ok(jobs) = with_database(|db| db.get_jobs()) {
                       ctx_job.set(jobs);
                    }
                    println!("Deleted job id: {}", id);
                }
                Ok(Err(e)) => println!("DB error: {:?}", e),
                Err(e) => println!("Task error: {:?}", e),
            }
        });
    }
    pub fn update_goal(&self, goal: GoalModel) {
        let mut ctx_goals: Signal<Vec<GoalModel>> = self.goals;
        let name = goal.name.clone();
        spawn(async move {
            let result = tokio::task::spawn_blocking(move || {
                with_database_mut(|db| {
                    if goal.id.is_some() {
                        println!("Update goal with name: {}", name);

                        db.update_goal(&goal)
                    } else {
                        println!("Saved goal with name: {}", name);

                        db.insert_goal(&goal).map(|_| ())
                    }
                })
            })
            .await;

            match result {
                Ok(Ok(id)) => {
                    if let Ok(goals) = with_database(|db| db.get_goals()) {
                       ctx_goals.set(goals);
                    }
                }
                Ok(Err(e)) => println!("DB error: {:?}", e),
                Err(e) => println!("Task error: {:?}", e),
            }
        });
    }
}

pub fn provide_app() {
    use_context_provider(|| AppProvider::default());
}

pub fn use_app() -> AppProvider {
    use_context::<AppProvider>()
}
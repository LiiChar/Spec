use dioxus::prelude::*;

use chrono::{DateTime, Local};

use crate::{
    core::{EventModel, GoalModel, JobModel, with_database, with_database_mut},
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
    pub goals: Signal<Vec<GoalModel>>,
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
        }
    }
}


impl AppProvider {
    pub fn update_jobs(&self, job: JobModel) {
        let mut ctx_job: Signal<Vec<JobModel>> = self.jobs;
        let name = job.name.clone();
        spawn(async move {
            let result = tokio::task::spawn_blocking(move || {
                with_database_mut(|db| {
                    if job.id.is_some() {
                        db.update_job(&job)
                    } else {
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
                    println!("Saved job id: {}", name);
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
                        db.update_goal(&goal)
                    } else {
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
                    println!("Saved job id: {}", name);
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
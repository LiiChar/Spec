use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct JobModel {
    pub name: String,
    pub description: Option<String>,
    pub def_start_ts: Option<i64>,
    pub def_end_ts: Option<i64>,
    pub start_ts: i64,
    pub end_ts: i64,
    pub proccess_path: Vec<Option<i64>>,
    pub cron: Option<String>,
    pub color: String,
}

impl JobModel {
    pub fn new(name: String, start_ts: i64, end_ts: i64, proccess_path: Vec<Option<i64>>) -> Self {
        Self {
            name,
            description: None,
            def_start_ts: None,
            def_end_ts: None,
            start_ts: start_ts,
            end_ts: end_ts,
            proccess_path: proccess_path,
            cron: None,
            color: "bg-secondary/20".to_string(),
        }
    }
}

impl Default for JobModel {
    fn default() -> Self {
        Self {
            name: "Job".to_string(),
            description: None,
            def_start_ts: None,
            def_end_ts: None,
            start_ts: 0,
            end_ts: 0,
            proccess_path: Vec::new(),
            cron: None,
            color: "bg-secondary/20".to_string(),
        }
    }
}

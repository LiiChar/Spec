use serde::{Deserialize, Serialize};

use super::TagModel;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum GoalOrder {
    Equal,
    Less,
    Greater,
}

impl GoalOrder {
    pub fn as_i32(self) -> i32 {
        match self {
            GoalOrder::Equal => 0,
            GoalOrder::Less => 1,
            GoalOrder::Greater => 2,
        }
    }

    pub fn from_i32(v: i32) -> Self {
        match v {
            1 => GoalOrder::Less,
            2 => GoalOrder::Greater,
            _ => GoalOrder::Equal,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            GoalOrder::Equal => "=",
            GoalOrder::Less => "<",
            GoalOrder::Greater => ">",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GoalModel {
    #[serde(default)]
    pub id: Option<i64>,
    pub name: String,
    pub description: Option<String>,
    pub ordering: GoalOrder,
    pub timestamp: i64,
    pub start_period_ts: i64,
    pub end_period_ts: i64,
    pub tags: Vec<TagModel>,
    pub process: String,
    pub completed: bool,
}

impl GoalModel {
    pub fn new(name: String, start_period_ts: i64, end_period_ts: i64, process: String) -> Self {
        Self {
            id: None,
            name,
            description: None,
            ordering: GoalOrder::Equal,
            timestamp: chrono::Utc::now().timestamp(),
            start_period_ts,
            end_period_ts,
            tags: Vec::new(),
            process,
            completed: false,
        }
    }
}

impl Default for GoalModel {
    fn default() -> Self {
        Self {
            id: None,
            name: "Цель".to_string(),
            description: None,
            ordering: GoalOrder::Equal,
            timestamp: 0,
            start_period_ts: 0,
            end_period_ts: 0,
            tags: Vec::new(),
            process: String::new(),
            completed: false,
        }
    }
}


#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Goal {
    pub id: i64,
    pub name: String,
    pub description: Option<String>,
    pub ordering: i64,
    pub timestamp: i64,
    pub start_period_ts: i64,
    pub end_period_ts: i64,
    pub process: String,
    pub completed: bool,
}

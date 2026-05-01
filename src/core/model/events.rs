use crate::core::WindowModel;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum EventType {
    Idle,
    WindowSwitch,
    Keyboard,
    Mouse,
}

impl EventType {
    pub fn as_i32(&self) -> i32 {
        match self {
            EventType::Idle => 0,
            EventType::WindowSwitch => 1,
            EventType::Keyboard => 2,
            EventType::Mouse => 3,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EventModel {
    pub window: Option<WindowModel>,
    pub event_type: EventType,
    pub timestamp: u64,
    pub duration: u64,
}

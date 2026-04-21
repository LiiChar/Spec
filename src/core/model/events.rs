use crate::core::WindowModel;

#[derive(Debug, Clone, PartialEq, Eq)]
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EventModel {
    pub window: Option<WindowModel>,
    pub event_type: EventType,
    pub timestamp: u64,
    pub duration: u64,
}
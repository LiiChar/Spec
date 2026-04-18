use crate::core::WindowModel;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum EventType {
    IDLE,
    WindowSwitch,
    KEYBOARD,
    MOUSE,
}

impl EventType {
    pub fn as_i32(&self) -> i32 {
        match self {
            EventType::IDLE => 0,
            EventType::WindowSwitch => 1,
            EventType::KEYBOARD => 2,
            EventType::MOUSE => 3,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct EventModel {
    pub window: WindowModel,
    pub event_type: EventType,
    pub timestamp: u64,
    pub duration: u64,
}
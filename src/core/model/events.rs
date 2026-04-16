use crate::core::WindowModel;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum EventType {
    IDLE,
    WindowSwitch,
    KEYBOARD,
    MOUSE,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct EventModel {
    pub window: WindowModel,
    pub event_type: EventType,
    pub timestamp: u64,
    pub duration: u64,
}
use serde::{Deserialize, Serialize};
use windows::Win32::Foundation::RECT;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Rect {
    pub left: i32,
    pub top: i32,
    pub right: i32,
    pub bottom: i32,
    pub width: i32,
    pub height: i32,
}

impl Rect {
    pub fn from_rect(rect: RECT) -> Self {
        Self {
            left: rect.left,
            top: rect.top,
            right: rect.right,
            bottom: rect.bottom,
            width: (rect.right - rect.left).max(0),
            height: (rect.bottom - rect.top).max(0),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WindowModel {
    pub hwnd: isize,

    pub title: String,
    pub class_name: String,
    pub icon_base64: Option<String>,

    pub process_name: String,
    pub process_path: String,
    pub pid: u32,

    pub rect: Rect,
    pub is_minimized: bool,
    pub is_maximized: bool,
    pub is_visible: bool,
    pub is_focused: bool,

    pub monitor_id: Option<u32>,

    pub timestamp: u64,
    pub duration: u64,
}

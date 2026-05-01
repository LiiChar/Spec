#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ToastAlign {
    LeftTop,
    LeftBottom,
    RightTop,
    RightBottom,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ToastType {
    Info,
    Success,
    Error,
}

#[derive(Debug, Clone, PartialEq, Eq)]

pub struct Toast {
    pub title: String,
    pub description: Option<String>,
    pub t: ToastType,
    pub timeout: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ToasterProvider {
    pub toasts: Vec<Toast>,
    pub align: ToastAlign,
}

impl Default for ToasterProvider {
    fn default() -> Self {
        Self {
            toasts: Vec::new(),
            align: ToastAlign::LeftBottom,
        }
    }
}

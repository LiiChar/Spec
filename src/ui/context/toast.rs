use dioxus_signals::{ReadableExt, Signal, WritableExt, WritableVecExt};

use crate::lib::time;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum ToastAlign {
    LeftTop,
    LeftBottom,
    RightTop,
    RightBottom,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum ToastType {
    Info,
    Success,
    Error,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]

pub struct Toast {
    pub title: String,
    pub description: Option<String>,
    pub t: ToastType,
    pub timeout: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ToasterProvider {
    pub toasts: Signal<Vec<Toast>>,
    pub align: ToastAlign,
}

impl Default for ToasterProvider {
    fn default() -> Self {
        Self {
            toasts: Signal::new(Vec::new()),
            align: ToastAlign::LeftBottom,
        }
    }
}

impl ToasterProvider {
    pub fn info(&mut self, title: String, description: Option<String>, timeout: Option<u32>) {
        let toast = Toast {
            title,
            description,
            t: ToastType::Info,
            timeout: timeout.unwrap_or(2000)
        };

        let mut temp = self.toasts.read().clone();
        temp.push(toast);
        self.toasts.set(temp);
    }
        pub fn success(&mut self, title: String, description: Option<String>, timeout: Option<u32>) {
        let toast = Toast {
            title,
            description,
            t: ToastType::Success,
            timeout: timeout.unwrap_or(2000)
        };

        let mut temp = self.toasts.read().clone();
        temp.push(toast);
        self.toasts.set(temp);
    }
        pub fn error(&mut self, title: String, description: Option<String>, timeout: Option<u32>) {
        let toast = Toast {
            title,
            description,
            t: ToastType::Error,
            timeout: timeout.unwrap_or(2000)
        };

        let mut temp = self.toasts.read().clone();
        temp.push(toast);
        self.toasts.set(temp);
    }
}
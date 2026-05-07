use std::{
    collections::{HashMap, VecDeque},
    sync::atomic::{AtomicU32, Ordering},
    time::Duration,
};

use dioxus::{core::{Task, spawn}, hooks::{use_context, use_context_provider}};
use dioxus_signals::{ReadableExt, Signal, WritableExt};
use tokio::{
    task::{AbortHandle},
    time::sleep,
};

static TOAST_ID: AtomicU32 = AtomicU32::new(1);

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
    pub id: u32,
    pub title: String,
    pub description: Option<String>,
    pub t: ToastType,
    pub timeout: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ToasterProvider {
    pub toasts: Signal<VecDeque<Toast>>,
    pub align: ToastAlign,
    pub aborts: Signal<HashMap<u32, Task>>,
}

impl Default for ToasterProvider {
    fn default() -> Self {
        Self {
            toasts: Signal::new(VecDeque::new()),
            align: ToastAlign::LeftBottom,
            aborts: Signal::new(HashMap::new()),
        }
    }
}

impl ToasterProvider {
    fn next_id() -> u32 {
        TOAST_ID.fetch_add(1, Ordering::Relaxed)
    }

    fn push(
        &mut self,
        title: String,
        description: Option<String>,
        t: ToastType,
        timeout: Option<u32>,
    ) {
        let timeout = timeout.unwrap_or(5000);
        let id = Self::next_id();

        let toast = Toast {
            id,
            title,
            description,
            t,
            timeout,
        };

        {
            let mut items = self.toasts.read().clone();
            items.push_back(toast);
            self.toasts.set(items);
        }

        let toasts = self.toasts;
        let mut aborts = self.aborts;

        let handle = spawn({
            let mut toasts = toasts;
            let mut aborts = aborts;

            async move {
                sleep(Duration::from_millis(timeout as u64)).await;

                let mut items = toasts.read().clone();
                items.retain(|t| t.id != id);
                toasts.set(items);

                let mut map = aborts.read().clone();
                map.remove(&id);
                aborts.set(map);
            }
        });

        {
            let mut map = aborts.read().clone();
            map.insert(id, handle);
            aborts.set(map);
        }
    }

    pub fn info(&mut self, title: String, description: Option<String>, timeout: Option<u32>) {
        self.push(title, description, ToastType::Info, timeout);
    }

    pub fn success(&mut self, title: String, description: Option<String>, timeout: Option<u32>) {
        self.push(title, description, ToastType::Success, timeout);
    }

    pub fn error(&mut self, title: String, description: Option<String>, timeout: Option<u32>) {
        self.push(title, description, ToastType::Error, timeout);
    }

    pub fn remove(&mut self, id: u32) {
        if let Some(task) = self.aborts.read().get(&id) {
            task.cancel();
        }

        {
            let mut map = self.aborts.read().clone();
            map.remove(&id);
            self.aborts.set(map);
        }

        {
            let mut items = self.toasts.read().clone();
            items.retain(|t| t.id != id);
            self.toasts.set(items);
        }
    }
}

pub fn provide_toast() {
    use_context_provider(|| ToasterProvider::default());
}

pub fn use_toast() -> ToasterProvider {
    use_context::<ToasterProvider>()
}
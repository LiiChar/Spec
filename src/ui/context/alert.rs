use std::{
    collections::{HashMap, VecDeque},
    sync::atomic::{AtomicU32, Ordering},
    time::Duration,
};

use dioxus::{
    core::{Callback, Task, spawn},
    hooks::{use_context, use_context_provider},
};
use dioxus_signals::{ReadableExt, Signal, WritableExt};
use tokio::{task::JoinHandle, time::sleep};

static ALERT_ID: AtomicU32 = AtomicU32::new(1);

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum AlertAlign {
    Top,
    Bottom,
    Right,
    Left,
    Center,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum AlertType {
    Info,
    Success,
    Error,
}

#[derive(Clone, PartialEq)]
pub struct Alert {
    pub id: u32,
    pub title: String,
    pub description: Option<String>,
    pub t: AlertType,
    pub timeout: u32,
    pub cb: Callback<bool>,
}

#[derive(Clone)]
pub struct AlertProvider {
    pub alerts: Signal<VecDeque<Alert>>,
    pub align: AlertAlign,
    pub aborts: Signal<HashMap<u32, Task>>,
}

impl Default for AlertProvider {
    fn default() -> Self {
        Self {
            alerts: Signal::new(VecDeque::new()),
            align: AlertAlign::Top,
            aborts: Signal::new(HashMap::new()),
        }
    }
}

impl AlertProvider {
    fn next_id() -> u32 {
        ALERT_ID.fetch_add(1, Ordering::Relaxed)
    }

    fn push(
        &mut self,
        title: String,
        description: Option<String>,
        t: AlertType,
        timeout: Option<u32>,
        cb: Callback<bool>,
    ) {
        let timeout = timeout.unwrap_or(5000);
        let id = Self::next_id();

        let alert = Alert {
            id,
            title,
            description,
            t,
            timeout,
            cb,
        };

        self.alerts.write().push_back(alert);

        let mut alerts = self.alerts;
        let mut aborts = self.aborts;

        let handle = spawn(async move {
            sleep(Duration::from_millis(timeout as u64)).await;

            alerts.write().retain(|a| a.id != id);
            aborts.write().remove(&id);
        });

        self.aborts.write().insert(id, handle);
    }

    pub fn info(
        &mut self,
        title: String,
        description: Option<String>,
        timeout: Option<u32>,
        cb: Callback<bool>,
    ) {
        self.push(title, description, AlertType::Info, timeout, cb);
    }

    pub fn success(
        &mut self,
        title: String,
        description: Option<String>,
        timeout: Option<u32>,
        cb: Callback<bool>,
    ) {
        self.push(title, description, AlertType::Success, timeout, cb);
    }

    pub fn error(
        &mut self,
        title: String,
        description: Option<String>,
        timeout: Option<u32>,
        cb: Callback<bool>,
    ) {
        self.push(title, description, AlertType::Error, timeout, cb);
    }

    fn dismiss(&mut self, id: u32, emit: Option<bool>) {
        if let Some(task) = self.aborts.write().remove(&id) {
            task.cancel();
        }

        let alert = {
            let alerts = self.alerts.read();
            alerts.iter().find(|a| a.id == id).cloned()
        };

        self.alerts.write().retain(|a| a.id != id);

        if let (Some(alert), Some(value)) = (alert, emit) {
            alert.cb.call(value);
        }
    }

    pub fn ok(&mut self, id: u32) {
        self.dismiss(id, Some(true));
    }

    pub fn cancel(&mut self, id: u32) {
        self.dismiss(id, Some(false));
    }

    pub fn remove(&mut self, id: u32) {
        self.dismiss(id, None);
    }

    pub fn clear(&mut self) {
        for (_, task) in self.aborts.write().drain() {
            task.cancel();
        }

        self.alerts.write().clear();
    }
}

pub fn provide_alert() {
    use_context_provider(|| AlertProvider::default());
}

pub fn use_alert() -> AlertProvider {
    use_context::<AlertProvider>()
}
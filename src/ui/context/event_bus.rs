use crossbeam_channel::Receiver;
use dioxus::prelude::*;

use crate::{core::EventModel, RX};

#[derive(Clone)]
pub struct EventBus(pub Receiver<EventModel>);

pub fn provide_event_bus() {
    use_context_provider(|| {
        let rx = RX
            .lock()
            .expect("RX lock failed")
            .as_ref()
            .expect("Event receiver not initialized")
            .clone();

        EventBus(rx)
    });
}

pub fn use_event_bus() -> EventBus {
    use_context::<EventBus>()
}

use std::sync::Arc;

use dioxus::prelude::*;

use crate::core::{Database, Db};
use crate::DB;

#[derive(Clone)]
pub struct DbContext(pub Db);

pub fn provide_db() {
    use_context_provider(|| DbContext(Arc::clone(&DB)));
}

pub fn use_db() -> DbContext {
    use_context::<DbContext>()
}

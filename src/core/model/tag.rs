use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TagModel {
    pub name: String,
    pub description: Option<String>,
    pub color: String,
}

impl TagModel {
    pub fn new(
        name: impl Into<String>,
        description: Option<&str>,
        color: impl Into<String>,
    ) -> Self {
        Self {
            name: name.into(),
            description: description.map(String::from),
            color: color.into(),
        }
    }
}

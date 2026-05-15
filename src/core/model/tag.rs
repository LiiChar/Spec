use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TagModel {
    #[serde(default)]
    pub id: Option<i64>,
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
            id: None,
            name: name.into(),
            description: description.map(String::from),
            color: color.into(),
        }
    }
}

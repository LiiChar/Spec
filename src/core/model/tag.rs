use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TagModel {
    #[serde(default)]
    pub id: Option<i64>,
    pub name: String,
    pub description: Option<String>,
    pub color: String,
    pub filter: Option<String>,
}

impl TagModel {
    pub fn new(
        name: impl Into<String>,
        description: Option<&str>,
        color: impl Into<String>,
        filter: Option<String>,
    ) -> Self {
        Self {
            id: None,
            name: name.into(),
            description: description.map(String::from),
            color: color.into(),
            filter,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum TagRuleField {
    Process,
    Title,
    BrowserUrl,
    Any,
}

impl Default for TagRuleField {
    fn default() -> Self {
        TagRuleField::Process
    }
}

impl TagRuleField {
    pub fn as_str(&self) -> &'static str {
        match self {
            TagRuleField::Process => "process",
            TagRuleField::Title => "title",
            TagRuleField::BrowserUrl => "browser",
            TagRuleField::Any => "any",
        }
    }

    pub fn from_str(value: &str) -> Self {
        match value.to_lowercase().as_str() {
            "process" => TagRuleField::Process,
            "title" => TagRuleField::Title,
            "browser" => TagRuleField::BrowserUrl,
            _ => TagRuleField::Any,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Eq)]
pub struct TagRule {
    pub field: TagRuleField,
    pub pattern: String,
    pub tag: String,
    pub enabled: bool,
}

impl Default for TagRule {
    fn default() -> Self {
        Self {
            field: TagRuleField::Process,
            pattern: String::new(),
            tag: String::new(),
            enabled: true,
        }
    }
}

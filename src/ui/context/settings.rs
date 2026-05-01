#[derive(Debug, Clone, PartialEq, Eq)]
enum Theme {
    Light,
    Dark,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SettingsProvider {
    pub theme: Theme,
}

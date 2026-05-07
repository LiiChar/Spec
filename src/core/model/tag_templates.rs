use super::TagModel;

pub struct AppTagPresetGroup {
    pub title: &'static str,
    pub hint: &'static str,
    pub tags: Vec<TagModel>,
}

pub fn app_tag_preset_groups() -> Vec<AppTagPresetGroup> {
    vec![
        AppTagPresetGroup {
            title: "Браузеры (Chrome, Edge, Firefox)",
            hint: "Веб, исследования, обучение",
            tags: vec![
                TagModel::new("веб", Some("серфинг и чтение"), "#60a5fa"),
                TagModel::new("исследование", Some("поиск и заметки"), "#38bdf8"),
                TagModel::new("видео", Some("стримы, YouTube"), "#f472b6"),
            ],
        },
        AppTagPresetGroup {
            title: "Редакторы кода (VS Code, Cursor, IDEA)",
            hint: "Разработка и ревью",
            tags: vec![
                TagModel::new("код", Some("разработка"), "#34d399"),
                TagModel::new("рефакторинг", None, "#2dd4bf"),
                TagModel::new("отладка", None, "#fbbf24"),
                TagModel::new("документация", Some("readme, комментарии"), "#a78bfa"),
            ],
        },
        AppTagPresetGroup {
            title: "Общение (Slack, Teams, Telegram, Discord)",
            hint: "Синхронное общение",
            tags: vec![
                TagModel::new("чат", Some("мессенджеры"), "#f97316"),
                TagModel::new("созвон", Some("голос/видео"), "#fb7185"),
                TagModel::new("почта", Some("Outlook, Thunderbird"), "#94a3b8"),
            ],
        },
        AppTagPresetGroup {
            title: "Дизайн (Figma, Photoshop, Blender)",
            hint: "Визуальная работа",
            tags: vec![
                TagModel::new("дизайн", Some("макеты UI"), "#e879f9"),
                TagModel::new("иллюстрация", None, "#c084fc"),
                TagModel::new("3d", None, "#818cf8"),
            ],
        },
        AppTagPresetGroup {
            title: "Игры и лаунчеры",
            hint: "Досуг",
            tags: vec![
                TagModel::new("игра", None, "#22c55e"),
                TagModel::new("стрим", Some("запись/трансляция"), "#ef4444"),
            ],
        },
        AppTagPresetGroup {
            title: "Офис (Excel, Word, Notion)",
            hint: "Документы и планирование",
            tags: vec![
                TagModel::new("таблицы", None, "#10b981"),
                TagModel::new("текст", Some("документы"), "#64748b"),
                TagModel::new("планирование", Some("задачи, календарь"), "#0ea5e9"),
            ],
        },
    ]
}

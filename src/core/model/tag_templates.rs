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
                TagModel::new(
                    "веб",
                    Some("серфинг и чтение"),
                    "#60a5fa",
                    Some(r"(?i)(chrome|msedge|firefox|opera|brave|zed)(?:[-_].*)?\.exe".to_string()),
                ),
                TagModel::new(
                    "исследование",
                    Some("поиск и заметки"),
                    "#38bdf8",
                    Some(r"(?i)(chrome|msedge|firefox|opera|brave|obsidian)(?:[-_].*)?\.exe".to_string()),
                ),
                TagModel::new(
                    "видео",
                    Some("стримы, YouTube"),
                    "#f472b6",
                    Some(r"(?i)(chrome|msedge|firefox|opera|brave|vlc)(?:[-_].*)?\.exe".to_string()),
                ),
                TagModel::new(
                    "Тайменеджмент",
                    None,
                    "#fff9bf",
                    Some(r"(?i)(spexe|timer)(?:[-_].*)?\.exe".to_string()),
                ),
            ],
        },
        AppTagPresetGroup {
            title: "Редакторы кода (VS Code, Cursor, IDEA)",
            hint: "Разработка и ревью",
            tags: vec![
                TagModel::new(
                    "код",
                    Some("разработка"),
                    "#34d399",
                    Some(r"(?i)(code|cursor|idea|pycharm|webstorm|clion|rustrover)(?:[-_].*)?\.exe".to_string()),
                ),
                TagModel::new(
                    "рефакторинг",
                    None,
                    "#2dd4bf",
                    Some(r"(?i)(code|cursor|idea|pycharm|webstorm|clion|rustrover)(?:[-_].*)?\.exe".to_string()),
                ),
                TagModel::new(
                    "отладка",
                    None,
                    "#fbbf24",
                    Some(r"(?i)(code|cursor|idea|pycharm|webstorm|clion|rustrover)(?:[-_].*)?\.exe".to_string()),
                ),
                TagModel::new(
                    "документация",
                    Some("readme, комментарии"),
                    "#a78bfa",
                    Some(r"(?i)(code|cursor|obsidian|notion)(?:[-_].*)?\.exe".to_string()),
                ),
            ],
        },
        AppTagPresetGroup {
            title: "Общение (Slack, Teams, Telegram, Discord)",
            hint: "Синхронное общение",
            tags: vec![
                TagModel::new(
                    "чат",
                    Some("мессенджеры"),
                    "#f97316",
                    Some(r"(?i)(telegram|discord|slack|teams)(?:[-_].*)?\.exe".to_string()),
                ),
                TagModel::new(
                    "созвон",
                    Some("голос/видео"),
                    "#fb7185",
                    Some(r"(?i)(teams|zoom|discord|skype)(?:[-_].*)?\.exe".to_string()),
                ),
                TagModel::new(
                    "почта",
                    Some("Outlook, Thunderbird"),
                    "#94a3b8",
                    Some(r"(?i)(outlook|thunderbird)(?:[-_].*)?\.exe".to_string()),
                ),
            ],
        },
        AppTagPresetGroup {
            title: "Дизайн (Figma, Photoshop, Blender)",
            hint: "Визуальная работа",
            tags: vec![
                TagModel::new(
                    "дизайн",
                    Some("макеты UI"),
                    "#e879f9",
                    Some(r"(?i)(figma|photoshop|photopea|xd)(?:[-_].*)?\.exe".to_string()),
                ),
                TagModel::new(
                    "иллюстрация",
                    None,
                    "#c084fc",
                    Some(r"(?i)(photoshop|illustrator|krita)(?:[-_].*)?\.exe".to_string()),
                ),
                TagModel::new(
                    "3d",
                    None,
                    "#818cf8",
                    Some(r"(?i)(blender|maya|3dsmax)(?:[-_].*)?\.exe".to_string()),
                ),
            ],
        },
        AppTagPresetGroup {
            title: "Игры и лаунчеры",
            hint: "Досуг",
            tags: vec![
                TagModel::new(
                    "игра",
                    None,
                    "#22c55e",
                    Some(r"(?i)(steam|epicgameslauncher|riotclient|leagueoflegends|dota2|cs2)(?:[-_].*)?\.exe".to_string()),
                ),
                TagModel::new(
                    "стрим",
                    Some("запись/трансляция"),
                    "#ef4444",
                    Some(r"(?i)(obs64|streamlabs)(?:[-_].*)?\.exe".to_string()),
                ),
            ],
        },
        AppTagPresetGroup {
            title: "Офис (Excel, Word, Notion)",
            hint: "Документы и планирование",
            tags: vec![
                TagModel::new(
                    "таблицы",
                    None,
                    "#10b981",
                    Some(r"(?i)(excel|libreofficecalc)(?:[-_].*)?\.exe".to_string()),
                ),
                TagModel::new(
                    "текст",
                    Some("документы"),
                    "#64748b",
                    Some(r"(?i)(winword|libreofficewriter|notion)(?:[-_].*)?\.exe".to_string()),
                ),
                TagModel::new(
                    "планирование",
                    Some("задачи, календарь"),
                    "#0ea5e5",
                    Some(r"(?i)(notion|todoist|outlook)(?:[-_].*)?\.exe".to_string()),
                ),
            ],
        },
    ]
}
use once_cell::sync::Lazy;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Mutex;
use dioxus::desktop::tao::window::Icon;

/// Кеш иконок: процесс -> base64 PNG
static ICON_CACHE: Lazy<Mutex<HashMap<String, String>>> = Lazy::new(|| Mutex::new(HashMap::new()));

/// Получить иконку приложения по имени процесса (возвращает base64 PNG)
pub fn get_app_icon(process_name: &str) -> Option<String> {
    // Проверяем кеш
    if let Ok(cache) = ICON_CACHE.lock() {
        if let Some(icon_data) = cache.get(process_name) {
            return Some(icon_data.clone());
        }
    }

    // Пытаемся найти и извлечь иконку
    if let Some(icon_data) = extract_app_icon(process_name) {
        // Сохраняем в кеш
        if let Ok(mut cache) = ICON_CACHE.lock() {
            cache.insert(process_name.to_string(), icon_data.clone());
        }
        return Some(icon_data);
    }

    None
}

/// Извлечь иконку из приложения
fn extract_app_icon(process_name: &str) -> Option<String> {
    // Обычные расположения приложений
    let possible_paths = vec![
        format!("C:\\Program Files\\{}\\{}.exe", process_name, process_name),
        format!(
            "C:\\Program Files (x86)\\{}\\{}.exe",
            process_name, process_name
        ),
        format!("C:\\Program Files\\Google\\Chrome\\Application\\chrome.exe"),
        format!("C:\\Program Files\\Mozilla Firefox\\firefox.exe"),
        format!("C:\\Program Files\\Microsoft VS Code\\Code.exe"),
    ];

    // Пытаемся найти исполняемый файл
    for path in possible_paths {
        if std::path::Path::new(&path).exists() {
            // Извлекаем иконку
            if let Some(icon_data) = try_extract_icon(&path) {
                return Some(icon_data);
            }
        }
    }

    // Fallback: возвращаем пустой квадрат как заглушку
    None
}

/// Попытка извлечь иконку из файла
fn try_extract_icon(path: &str) -> Option<String> {
    // TODO: Реализовать правильное извлечение иконки из PE файла
    // Это сложная задача, требует парсинга PE формата
    // Для теперь возвращаем None
    None
}

/// Получить цвет для процесса (детерминированный на основе названия)
pub fn get_process_color(process_name: &str) -> &'static str {
    let name = process_name.to_lowercase();

    for (pattern, color) in PROCESS_COLORS.iter() {
        if name.contains(pattern) {
            return color;
        }
    }

    "bg-zinc-500"
}

static PROCESS_COLORS: &[(&str, &str)] = &[
    // ---------------- Browsers ----------------
    ("firefox", "bg-orange-500"),
    ("chrome", "bg-blue-500"),
    ("edge", "bg-sky-500"),
    ("opera", "bg-red-500"),
    ("brave", "bg-orange-600"),
    ("vivaldi", "bg-red-400"),
    ("zen", "bg-gray-500"),
    // ---------------- Dev tools ----------------
    ("code", "bg-blue-600"),
    ("vscode", "bg-blue-600"),
    ("visual studio", "bg-purple-700"),
    ("idea", "bg-red-500"),
    ("pycharm", "bg-green-500"),
    ("webstorm", "bg-cyan-500"),
    ("phpstorm", "bg-indigo-500"),
    ("goland", "bg-sky-600"),
    ("rider", "bg-rose-500"),
    ("docker", "bg-blue-400"),
    ("postman", "bg-orange-500"),
    ("gitkraken", "bg-purple-500"),
    ("github", "bg-gray-700"),
    ("git", "bg-gray-600"),
    // ---------------- Communication ----------------
    ("discord", "bg-indigo-500"),
    ("slack", "bg-purple-500"),
    ("telegram", "bg-sky-500"),
    ("whatsapp", "bg-green-500"),
    ("zoom", "bg-blue-400"),
    ("teams", "bg-indigo-600"),
    ("skype", "bg-blue-500"),
    ("viber", "bg-purple-600"),
    // ---------------- Office ----------------
    ("word", "bg-blue-700"),
    ("excel", "bg-green-600"),
    ("powerpoint", "bg-orange-600"),
    ("outlook", "bg-blue-500"),
    ("notion", "bg-gray-800"),
    ("onenote", "bg-purple-500"),
    ("libreoffice", "bg-blue-800"),
    // ---------------- Editors ----------------
    ("notepad", "bg-gray-500"),
    ("sublime", "bg-orange-500"),
    ("atom", "bg-green-600"),
    ("obsidian", "bg-purple-600"),
    ("vim", "bg-green-700"),
    ("neovim", "bg-green-800"),
    // ---------------- System ----------------
    ("explorer", "bg-yellow-600"),
    ("finder", "bg-yellow-500"),
    ("settings", "bg-gray-600"),
    ("control panel", "bg-gray-700"),
    // ---------------- Terminal ----------------
    ("terminal", "bg-slate-800"),
    ("powershell", "bg-blue-900"),
    ("cmd", "bg-gray-900"),
    ("bash", "bg-gray-800"),
    ("zsh", "bg-gray-900"),
    // ---------------- Media ----------------
    ("spotify", "bg-green-500"),
    ("vlc", "bg-orange-500"),
    ("mpv", "bg-red-600"),
    ("youtube", "bg-red-500"),
    ("netflix", "bg-red-700"),
    ("twitch", "bg-purple-600"),
    ("obs", "bg-gray-700"),
    ("audacity", "bg-blue-600"),
    // ---------------- Design ----------------
    ("figma", "bg-pink-500"),
    ("photoshop", "bg-blue-900"),
    ("illustrator", "bg-orange-700"),
    ("after effects", "bg-purple-800"),
    ("blender", "bg-orange-600"),
    // ---------------- Game launchers ----------------
    ("steam", "bg-slate-700"),
    ("epic", "bg-gray-800"),
    ("battle.net", "bg-blue-700"),
    ("battlenet", "bg-blue-700"),
    ("riot", "bg-red-600"),
    ("riot client", "bg-red-600"),
    ("origin", "bg-orange-600"),
    ("ea app", "bg-orange-700"),
    ("ubisoft", "bg-blue-600"),
    ("uplay", "bg-blue-600"),
    ("gog", "bg-purple-700"),
    ("xbox", "bg-green-600"),
    // ---------------- Games (Valve / FPS / MOBA) ----------------
    ("cs2", "bg-yellow-500"),
    ("csgo", "bg-yellow-500"),
    ("counter-strike", "bg-yellow-500"),
    ("dota", "bg-red-700"),
    ("dota 2", "bg-red-700"),
    ("half-life", "bg-orange-600"),
    // Riot games
    ("league of legends", "bg-blue-700"),
    ("lol", "bg-blue-700"),
    ("valorant", "bg-red-500"),
    ("teamfight tactics", "bg-purple-600"),
    // Blizzard
    ("overwatch", "bg-orange-500"),
    ("wow", "bg-blue-800"),
    ("warcraft", "bg-blue-800"),
    ("diablo", "bg-red-800"),
    // AAA games
    ("fortnite", "bg-indigo-500"),
    ("minecraft", "bg-green-700"),
    ("gta", "bg-emerald-600"),
    ("cyberpunk", "bg-yellow-400"),
    ("elden ring", "bg-yellow-600"),
    ("witcher", "bg-red-700"),
    ("skyrim", "bg-gray-400"),
    ("starfield", "bg-sky-700"),
    ("apex", "bg-red-600"),
    ("pubg", "bg-yellow-600"),
    // FPS
    ("call of duty", "bg-gray-700"),
    ("cod", "bg-gray-700"),
    ("battlefield", "bg-blue-500"),
    ("rainbow six", "bg-orange-600"),
    ("r6", "bg-orange-600"),
    // Indie
    ("terraria", "bg-green-600"),
    ("stardew", "bg-lime-500"),
    ("hades", "bg-red-500"),
    ("dead cells", "bg-purple-700"),
    ("hollow knight", "bg-gray-800"),
    ("factorio", "bg-orange-600"),
    // Engines
    ("unity", "bg-gray-700"),
    ("unreal", "bg-black"),
    ("godot", "bg-sky-500"),
    // ---------------- Extra apps (≈30+) ----------------
    ("dropbox", "bg-blue-500"),
    ("onedrive", "bg-blue-600"),
    ("google drive", "bg-green-500"),
    ("zoominfo", "bg-indigo-600"),
    ("figjam", "bg-pink-400"),
    ("slackbot", "bg-purple-400"),
    ("jira", "bg-blue-700"),
    ("confluence", "bg-blue-800"),
    ("trello", "bg-sky-600"),
    ("asana", "bg-red-500"),
    ("monday", "bg-pink-600"),
    ("notion calendar", "bg-gray-700"),
    ("calendly", "bg-blue-500"),
    ("calendar", "bg-indigo-400"),
    ("photos", "bg-cyan-500"),
    ("paint", "bg-cyan-500"),
    ("gimp", "bg-orange-400"),
    ("inkscape", "bg-indigo-500"),
    ("lightroom", "bg-blue-800"),
    ("davinci resolve", "bg-black"),
    ("handbrake", "bg-red-600"),
    ("winrar", "bg-purple-700"),
    ("7zip", "bg-gray-600"),
    ("bluestacks", "bg-blue-400"),
    ("android studio", "bg-green-700"),
    ("xcode", "bg-gray-900"),
    ("docker desktop", "bg-blue-500"),
    ("postbird", "bg-yellow-600"),
    ("dbeaver", "bg-orange-500"),
    ("pgadmin", "bg-blue-700"),
];

pub fn get_process_color_gradient(process_name: &str) -> &'static str {
    match process_name.to_lowercase().as_str() {
        // Browsers
        s if s.contains("firefox") => "from-orange-500 to-red-400",
        s if s.contains("chrome") => "from-blue-500 to-cyan-400",
        s if s.contains("edge") => "from-sky-500 to-teal-400",
        s if s.contains("opera") => "from-red-600 to-pink-500",
        s if s.contains("brave") => "from-orange-600 to-red-500",

        // Dev
        s if s.contains("code") || s.contains("vscode") => "from-blue-600 to-indigo-500",
        s if s.contains("visual studio") => "from-purple-700 to-indigo-600",
        s if s.contains("idea") => "from-red-600 to-pink-500",
        s if s.contains("pycharm") => "from-green-600 to-emerald-400",
        s if s.contains("webstorm") => "from-cyan-600 to-blue-500",
        s if s.contains("rider") => "from-rose-600 to-pink-500",
        s if s.contains("docker") => "from-blue-500 to-cyan-400",
        s if s.contains("postman") => "from-orange-500 to-amber-400",

        // Communication
        s if s.contains("slack") => "from-purple-500 to-pink-400",
        s if s.contains("discord") => "from-indigo-600 to-purple-500",
        s if s.contains("telegram") => "from-sky-500 to-blue-400",
        s if s.contains("whatsapp") => "from-green-500 to-emerald-400",
        s if s.contains("zoom") => "from-blue-400 to-indigo-400",
        s if s.contains("teams") => "from-indigo-600 to-violet-500",

        // Office
        s if s.contains("word") => "from-blue-700 to-blue-500",
        s if s.contains("excel") => "from-green-600 to-emerald-400",
        s if s.contains("powerpoint") => "from-orange-600 to-red-400",
        s if s.contains("outlook") => "from-blue-500 to-sky-400",
        s if s.contains("notion") => "from-gray-800 to-gray-600",

        // Media
        s if s.contains("spotify") => "from-green-500 to-lime-400",
        s if s.contains("vlc") => "from-orange-500 to-amber-400",
        s if s.contains("youtube") => "from-red-600 to-red-400",

        // Design
        s if s.contains("figma") => "from-pink-500 to-orange-400",
        s if s.contains("photoshop") => "from-blue-900 to-indigo-700",
        s if s.contains("illustrator") => "from-orange-700 to-yellow-500",
        s if s.contains("after effects") => "from-purple-800 to-indigo-600",

        // Terminal
        s if s.contains("terminal") || s.contains("powershell") || s.contains("cmd") => {
            "from-slate-800 to-slate-700"
        }

        _ => "from-zinc-600 to-zinc-400",
    }
}


pub fn load_icon(path: &std::path::Path) -> Icon {
    let (icon_rgba, icon_width, icon_height) = {
        let image = image::open(path)
            .expect("Failed to open icon path")
            .into_rgba8();
        let (width, height) = image.dimensions();
        let rgba = image.into_raw();
        (rgba, width, height)
    };
    Icon::from_rgba(icon_rgba, icon_width, icon_height).expect("Failed to open icon")
}
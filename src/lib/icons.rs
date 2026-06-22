use image::EncodableLayout;
use once_cell::sync::Lazy;
use std::collections::HashMap;
use std::io::Read;
use std::path::PathBuf;
use std::sync::Mutex;
use dioxus::desktop::tao::window::Icon;
use std::fs::File;
use dioxus::prelude::*;
use color_thief::ColorFormat;
use base64::Engine;
use crate::core::EventModel;

/// Кеш иконок: процесс -> base64 PNG
static ICON_CACHE: Lazy<Mutex<HashMap<String, String>>> = Lazy::new(|| Mutex::new(HashMap::new()));
const ICONS_DIR: Asset = asset!("/assets/icons");

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

pub fn extract_icon_events(events: Vec<EventModel>) -> Vec<EventModel> {
    events
        .into_iter()
        .map(|mut event| {
            if let Some(mut window) = event.window {
                window.icon_base64 = extract_icon(window.icon_base64.unwrap_or_default());
                event.window = Some(window);
            }
            event
        })
        .collect()
}

pub fn extract_icon(path: String) -> Option<String> {
    let trimmed = path.trim();

    if trimmed.is_empty() {
        return default_icon_data_uri();
    }

    if trimmed.starts_with("data:image/png;base64,") {
        return Some(trimmed.to_string());
    }

    let icon_path = PathBuf::from(trimmed);
    if icon_path.exists() {
        if let Ok(mut file) = File::open(&icon_path) {
            let mut content = String::new();
            if file.read_to_string(&mut content).is_ok() {
                return Some(format!("data:image/png;base64,{}", content.trim()));
            }
        }
    }

    let icon_dir = PathBuf::from(ICONS_DIR.bundled().absolute_source_path());
    let icon_file = icon_dir.join(trimmed);
    if icon_file.exists() {
        if let Ok(mut file) = File::open(&icon_file) {
            let mut content = String::new();
            if file.read_to_string(&mut content).is_ok() {
                return Some(format!("data:image/png;base64,{}", content.trim()));
            }
        }
    }

    if looks_like_base64(trimmed) {
        return Some(format!("data:image/png;base64,{}", trimmed));
    }

    default_icon_data_uri()
}

fn looks_like_base64(value: &str) -> bool {
    let trimmed = value.trim();
    trimmed.len() > 32 && trimmed.chars().all(|c| {
        c.is_ascii_alphanumeric() || c == '+' || c == '/' || c == '=' || c == '\n' || c == '\r'
    })
}

fn default_icon_data_uri() -> Option<String> {
    let icon_dir = PathBuf::from(ICONS_DIR.bundled().absolute_source_path()).join("default.exe.txt");
    if let Ok(mut file) = File::open(&icon_dir) {
        let mut content = String::new();
        if file.read_to_string(&mut content).is_ok() {
            return Some(format!("data:image/png;base64,{}", content.trim()));
        }
    }
    None
}

// src/lib/icons.rs
pub fn get_primary_icon_color(icon: String, process_name: String) -> String {
    let base64_data = if icon.contains(";base64,") {
        icon.split(";base64,").nth(1).unwrap_or(&icon).to_string()
    } else {
        icon.clone()
    };

    let color_from_icon = (|| -> Option<String> {
        let decoded = base64::engine::general_purpose::STANDARD
            .decode(&base64_data)
            .ok()?;
        let img = image::load_from_memory(&decoded).ok()?;
        let rgba = img.to_rgba8();
        let raw_pixels = rgba.as_raw();
        let colors = color_thief::get_palette(raw_pixels, ColorFormat::Rgba, 6, 10).ok()?;
        colors.first().map(|v| v.to_string())
    })();

    color_from_icon.unwrap_or_else(|| get_process_color(&process_name).to_string())
}

fn extract_app_icon(process_name: &str) -> Option<String> {
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
    ("firefox", "rgba(249, 115, 22, 1)"),     // orange-500
    ("chrome", "rgba(59, 130, 246, 1)"),      // blue-500
    ("edge", "rgba(14, 165, 233, 1)"),        // sky-500
    ("opera", "rgba(239, 68, 68, 1)"),        // red-500
    ("brave", "rgba(234, 88, 12, 1)"),        // orange-600
    ("vivaldi", "rgba(248, 113, 113, 1)"),    // red-400
    ("zen", "rgba(107, 114, 128, 1)"),        // gray-500
    // ---------------- Dev tools ----------------
    ("code", "rgba(37, 99, 235, 1)"),         // blue-600
    ("vscode", "rgba(37, 99, 235, 1)"),
    ("visual studio", "rgba(126, 34, 206, 1)"), // purple-700
    ("idea", "rgba(239, 68, 68, 1)"),
    ("pycharm", "rgba(34, 197, 94, 1)"),      // green-500
    ("webstorm", "rgba(6, 182, 212, 1)"),     // cyan-500
    ("phpstorm", "rgba(99, 102, 241, 1)"),    // indigo-500
    ("goland", "rgba(2, 132, 199, 1)"),       // sky-600
    ("rider", "rgba(244, 63, 94, 1)"),        // rose-500
    ("docker", "rgba(96, 165, 250, 1)"),      // blue-400
    ("postman", "rgba(249, 115, 22, 1)"),
    ("gitkraken", "rgba(168, 85, 247, 1)"),   // purple-500
    ("github", "rgba(55, 65, 81, 1)"),        // gray-700
    ("git", "rgba(75, 85, 99, 1)"),           // gray-600
    // ---------------- Communication ----------------
    ("discord", "rgba(99, 102, 241, 1)"),
    ("slack", "rgba(168, 85, 247, 1)"),
    ("telegram", "rgba(14, 165, 233, 1)"),
    ("whatsapp", "rgba(34, 197, 94, 1)"),
    ("zoom", "rgba(96, 165, 250, 1)"),
    ("teams", "rgba(79, 70, 229, 1)"),        // indigo-600
    ("skype", "rgba(59, 130, 246, 1)"),
    ("viber", "rgba(147, 51, 234, 1)"),       // purple-600
    // ---------------- Office ----------------
    ("word", "rgba(29, 78, 216, 1)"),         // blue-700
    ("excel", "rgba(22, 163, 74, 1)"),        // green-600
    ("powerpoint", "rgba(234, 88, 12, 1)"),
    ("outlook", "rgba(59, 130, 246, 1)"),
    ("notion", "rgba(31, 41, 55, 1)"),        // gray-800
    ("onenote", "rgba(168, 85, 247, 1)"),
    ("libreoffice", "rgba(30, 64, 175, 1)"),  // blue-800
    // ---------------- Editors ----------------
    ("notepad", "rgba(107, 114, 128, 1)"),
    ("sublime", "rgba(249, 115, 22, 1)"),
    ("atom", "rgba(22, 163, 74, 1)"),
    ("obsidian", "rgba(147, 51, 234, 1)"),
    ("vim", "rgba(21, 128, 61, 1)"),          // green-700
    ("neovim", "rgba(22, 101, 52, 1)"),       // green-800
    // ---------------- System ----------------
    ("explorer", "rgba(202, 138, 4, 1)"),     // yellow-600
    ("finder", "rgba(234, 179, 8, 1)"),       // yellow-500
    ("settings", "rgba(75, 85, 99, 1)"),
    ("control panel", "rgba(55, 65, 81, 1)"),
    // ---------------- Terminal ----------------
    ("terminal", "rgba(30, 41, 59, 1)"),      // slate-800
    ("powershell", "rgba(30, 58, 138, 1)"),   // blue-900
    ("cmd", "rgba(17, 24, 39, 1)"),           // gray-900
    ("bash", "rgba(31, 41, 55, 1)"),
    ("zsh", "rgba(17, 24, 39, 1)"),
    // ---------------- Media ----------------
    ("spotify", "rgba(34, 197, 94, 1)"),
    ("vlc", "rgba(249, 115, 22, 1)"),
    ("mpv", "rgba(220, 38, 38, 1)"),          // red-600
    ("youtube", "rgba(239, 68, 68, 1)"),
    ("netflix", "rgba(185, 28, 28, 1)"),      // red-700
    ("twitch", "rgba(147, 51, 234, 1)"),
    ("obs", "rgba(55, 65, 81, 1)"),
    ("audacity", "rgba(37, 99, 235, 1)"),
    // ---------------- Design ----------------
    ("figma", "rgba(236, 72, 153, 1)"),       // pink-500
    ("photoshop", "rgba(30, 58, 138, 1)"),
    ("illustrator", "rgba(194, 65, 12, 1)"),  // orange-700
    ("after effects", "rgba(126, 34, 206, 1)"),
    ("blender", "rgba(234, 88, 12, 1)"),
    // ---------------- Game launchers ----------------
    ("steam", "rgba(51, 65, 85, 1)"),         // slate-700
    ("epic", "rgba(31, 41, 55, 1)"),
    ("battle.net", "rgba(29, 78, 216, 1)"),
    ("battlenet", "rgba(29, 78, 216, 1)"),
    ("riot", "rgba(220, 38, 38, 1)"),
    ("riot client", "rgba(220, 38, 38, 1)"),
    ("origin", "rgba(234, 88, 12, 1)"),
    ("ea app", "rgba(194, 65, 12, 1)"),
    ("ubisoft", "rgba(37, 99, 235, 1)"),
    ("uplay", "rgba(37, 99, 235, 1)"),
    ("gog", "rgba(126, 34, 206, 1)"),
    ("xbox", "rgba(22, 163, 74, 1)"),
    // ---------------- Games (Valve / FPS / MOBA) ----------------
    ("cs2", "rgba(234, 179, 8, 1)"),
    ("csgo", "rgba(234, 179, 8, 1)"),
    ("counter-strike", "rgba(234, 179, 8, 1)"),
    ("dota", "rgba(185, 28, 28, 1)"),
    ("dota 2", "rgba(185, 28, 28, 1)"),
    ("half-life", "rgba(234, 88, 12, 1)"),
    ("league of legends", "rgba(29, 78, 216, 1)"),
    ("lol", "rgba(29, 78, 216, 1)"),
    ("valorant", "rgba(239, 68, 68, 1)"),
    ("teamfight tactics", "rgba(147, 51, 234, 1)"),
    ("overwatch", "rgba(249, 115, 22, 1)"),
    ("wow", "rgba(30, 64, 175, 1)"),          // blue-800
    ("warcraft", "rgba(30, 64, 175, 1)"),
    ("diablo", "rgba(153, 27, 27, 1)"),       // red-800
    ("fortnite", "rgba(99, 102, 241, 1)"),
    ("minecraft", "rgba(21, 128, 61, 1)"),
    ("gta", "rgba(5, 150, 105, 1)"),          // emerald-600
    ("cyberpunk", "rgba(250, 204, 21, 1)"),   // yellow-400
    ("elden ring", "rgba(202, 138, 4, 1)"),
    ("witcher", "rgba(185, 28, 28, 1)"),
    ("skyrim", "rgba(156, 163, 175, 1)"),     // gray-400
    ("starfield", "rgba(3, 105, 161, 1)"),    // sky-700
    ("apex", "rgba(220, 38, 38, 1)"),
    ("pubg", "rgba(202, 138, 4, 1)"),
    ("call of duty", "rgba(55, 65, 81, 1)"),
    ("cod", "rgba(55, 65, 81, 1)"),
    ("battlefield", "rgba(59, 130, 246, 1)"),
    ("rainbow six", "rgba(234, 88, 12, 1)"),
    ("r6", "rgba(234, 88, 12, 1)"),
    ("terraria", "rgba(22, 163, 74, 1)"),
    ("stardew", "rgba(132, 204, 22, 1)"),     // lime-500
    ("hades", "rgba(239, 68, 68, 1)"),
    ("dead cells", "rgba(126, 34, 206, 1)"),
    ("hollow knight", "rgba(31, 41, 55, 1)"),
    ("factorio", "rgba(234, 88, 12, 1)"),
    ("unity", "rgba(55, 65, 81, 1)"),
    ("unreal", "rgba(0, 0, 0, 1)"),
    ("godot", "rgba(14, 165, 233, 1)"),
    // ---------------- Extra apps ----------------
    ("dropbox", "rgba(59, 130, 246, 1)"),
    ("onedrive", "rgba(37, 99, 235, 1)"),
    ("google drive", "rgba(34, 197, 94, 1)"),
    ("zoominfo", "rgba(79, 70, 229, 1)"),
    ("figjam", "rgba(244, 114, 182, 1)"),     // pink-400
    ("slackbot", "rgba(192, 132, 252, 1)"),   // purple-400
    ("jira", "rgba(29, 78, 216, 1)"),
    ("confluence", "rgba(30, 64, 175, 1)"),
    ("trello", "rgba(2, 132, 199, 1)"),
    ("asana", "rgba(239, 68, 68, 1)"),
    ("monday", "rgba(219, 39, 119, 1)"),      // pink-600
    ("notion calendar", "rgba(55, 65, 81, 1)"),
    ("calendly", "rgba(59, 130, 246, 1)"),
    ("calendar", "rgba(129, 140, 248, 1)"),   // indigo-400
    ("photos", "rgba(6, 182, 212, 1)"),
    ("paint", "rgba(6, 182, 212, 1)"),
    ("gimp", "rgba(251, 146, 60, 1)"),        // orange-400
    ("inkscape", "rgba(99, 102, 241, 1)"),
    ("lightroom", "rgba(30, 64, 175, 1)"),
    ("davinci resolve", "rgba(0, 0, 0, 1)"),
    ("handbrake", "rgba(220, 38, 38, 1)"),
    ("winrar", "rgba(126, 34, 206, 1)"),
    ("7zip", "rgba(75, 85, 99, 1)"),
    ("bluestacks", "rgba(96, 165, 250, 1)"),
    ("android studio", "rgba(21, 128, 61, 1)"),
    ("xcode", "rgba(17, 24, 39, 1)"),
    ("docker desktop", "rgba(59, 130, 246, 1)"),
    ("postbird", "rgba(202, 138, 4, 1)"),
    ("dbeaver", "rgba(249, 115, 22, 1)"),
    ("pgadmin", "rgba(29, 78, 216, 1)"),
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
use std::collections::HashMap;
use std::sync::Mutex;
use once_cell::sync::Lazy;
use std::path::PathBuf;

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
        format!("C:\\Program Files (x86)\\{}\\{}.exe", process_name, process_name),
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
    match process_name.to_lowercase().as_str() {
        // Browsers
        s if s.contains("firefox") => "bg-orange-500",
        s if s.contains("chrome") => "bg-blue-500",
        s if s.contains("edge") => "bg-sky-500",
        s if s.contains("opera") => "bg-red-500",
        s if s.contains("brave") => "bg-orange-600",
        s if s.contains("zen") => "bg-gray-500",

        // Dev
        s if s.contains("code") || s.contains("vscode") => "bg-blue-600",
        s if s.contains("visual studio") => "bg-purple-700",
        s if s.contains("idea") => "bg-red-500",
        s if s.contains("pycharm") => "bg-green-500",
        s if s.contains("webstorm") => "bg-cyan-500",
        s if s.contains("rider") => "bg-rose-500",
        s if s.contains("goland") => "bg-sky-600",
        s if s.contains("docker") => "bg-blue-400",
        s if s.contains("postman") => "bg-orange-500",

        // Communication
        s if s.contains("slack") => "bg-purple-500",
        s if s.contains("discord") => "bg-indigo-500",
        s if s.contains("telegram") => "bg-sky-500",
        s if s.contains("whatsapp") => "bg-green-500",
        s if s.contains("zoom") => "bg-blue-400",
        s if s.contains("teams") => "bg-indigo-600",

        // Office
        s if s.contains("word") => "bg-blue-700",
        s if s.contains("excel") => "bg-green-600",
        s if s.contains("powerpoint") => "bg-orange-600",
        s if s.contains("outlook") => "bg-blue-500",
        s if s.contains("notion") => "bg-gray-800",

        // Editors
        s if s.contains("notepad") => "bg-gray-500",
        s if s.contains("sublime") => "bg-orange-500",
        s if s.contains("obsidian") => "bg-purple-600",

        // System / file
        s if s.contains("explorer") || s.contains("finder") => "bg-yellow-600",
        s if s.contains("settings") => "bg-gray-600",

        // Terminal
        s if s.contains("terminal") || s.contains("powershell") || s.contains("cmd") => "bg-slate-800",
        s if s.contains("bash") || s.contains("zsh") => "bg-gray-900",

        // Media
        s if s.contains("spotify") => "bg-green-500",
        s if s.contains("vlc") => "bg-orange-500",
        s if s.contains("mpv") => "bg-red-600",
        s if s.contains("youtube") => "bg-red-500",

        // Design
        s if s.contains("figma") => "bg-pink-500",
        s if s.contains("photoshop") => "bg-blue-900",
        s if s.contains("illustrator") => "bg-orange-700",
        s if s.contains("after effects") => "bg-purple-800",

        // Images
        s if s.contains("photo") || s.contains("image") || s.contains("paint") => "bg-cyan-500",

        _ => "bg-zinc-500",
    }
}

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
        s if s.contains("terminal") || s.contains("powershell") || s.contains("cmd") => "from-slate-800 to-slate-700",

        _ => "from-zinc-600 to-zinc-400",
    }
}
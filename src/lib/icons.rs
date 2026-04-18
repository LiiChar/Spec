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
        s if s.contains("firefox") => "bg-orange-500",
        s if s.contains("chrome") => "bg-blue-500",
        s if s.contains("code") || s.contains("vscode") => "bg-purple-500",
        s if s.contains("visual") => "bg-purple-600",
        s if s.contains("slack") || s.contains("telegram") => "bg-pink-500",
        s if s.contains("discord") => "bg-indigo-500",
        s if s.contains("idea") || s.contains("pycharm") || s.contains("rider") => "bg-red-500",
        s if s.contains("word") || s.contains("excel") || s.contains("outlook") => "bg-green-500",
        s if s.contains("notepad") || s.contains("sublime") => "bg-yellow-500",
        s if s.contains("explorer") || s.contains("finder") => "bg-gray-500",
        s if s.contains("terminal") || s.contains("powershell") || s.contains("cmd") => "bg-slate-700",
        s if s.contains("photo") || s.contains("image") || s.contains("paint") => "bg-cyan-500",
        s if s.contains("video") || s.contains("vlc") || s.contains("mpv") => "bg-red-600",
        _ => "bg-indigo-400",
    }
}

pub fn get_process_color_gradient(process_name: &str) -> &'static str {
    match process_name.to_lowercase().as_str() {
        s if s.contains("firefox") => "from-orange-600 to-orange-400",
        s if s.contains("chrome") => "from-blue-600 to-blue-400",
        s if s.contains("code") || s.contains("vscode") => "from-purple-600 to-purple-400",
        s if s.contains("visual") => "from-purple-700 to-purple-500",
        s if s.contains("slack") || s.contains("telegram") => "from-pink-600 to-pink-400",
        s if s.contains("discord") => "from-indigo-600 to-indigo-400",
        s if s.contains("idea") || s.contains("pycharm") || s.contains("rider") => "from-red-600 to-red-400",
        s if s.contains("word") || s.contains("excel") || s.contains("outlook") => "from-green-600 to-green-400",
        s if s.contains("notepad") || s.contains("sublime") => "from-yellow-600 to-yellow-400",
        s if s.contains("explorer") || s.contains("finder") => "from-gray-600 to-gray-400",
        s if s.contains("terminal") || s.contains("powershell") || s.contains("cmd") => "from-slate-700 to-slate-600",
        s if s.contains("photo") || s.contains("image") || s.contains("paint") => "from-cyan-600 to-cyan-400",
        s if s.contains("video") || s.contains("vlc") || s.contains("mpv") => "from-red-700 to-red-500",
        _ => "from-indigo-500 to-indigo-300",
    }
}

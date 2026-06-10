use palette::{Hsl, Srgba};

fn parse_rgba(color: &str) -> Option<(f32, f32, f32, f32)> {
    let c = color.trim();

    if c.starts_with("rgba") || c.starts_with("rgb") {
        let start = c.find('(')?;
        let end = c.find(')')?;

        let parts: Vec<f32> = c[start + 1..end]
            .split(',')
            .map(|v| v.trim().parse().ok())
            .collect::<Option<Vec<_>>>()?;

        return Some((
            parts[0] / 255.0,
            parts[1] / 255.0,
            parts[2] / 255.0,
            parts.get(3).copied().unwrap_or(1.0),
        ));
    }

    None
}

pub fn set_alpha(color: &str, alpha: f32) -> String {
    let (r, g, b, _) = parse_rgba(color)
        .unwrap_or((0.0, 0.0, 0.0, 1.0));

    let mut c = Srgba::new(r, g, b, 1.0);
    c.alpha = alpha;

    format!(
        "rgba({}, {}, {}, {})",
        (c.red * 255.0) as u8,
        (c.green * 255.0) as u8,
        (c.blue * 255.0) as u8,
        c.alpha
    )
}

use palette::{Srgb, IntoColor, Hsv};

pub fn soften_color(color: &str, strength: f32) -> String {
    let (r, g, b, _) = parse_rgba(color)
        .unwrap_or((0.0, 0.0, 0.0, 1.0));

    let rgb = Srgb::new(r, g, b);
    let mut hsv: Hsv = rgb.into_color();

    hsv.saturation *= 0.6; // приглушение
    hsv.value = hsv.value.powf(0.9); // мягкая компрессия яркости

    let rgb2: Srgb = hsv.into_color();

    format!(
        "rgba({}, {}, {}, {})",
        (rgb2.red * 255.0) as u8,
        (rgb2.green * 255.0) as u8,
        (rgb2.blue * 255.0) as u8,
        1.0
    )
}

pub fn idle_color(color: &str) -> String {
    let (r, g, b, _) = parse_rgba(color)
        .unwrap_or((0.0, 0.0, 0.0, 1.0));

    let rgb = Srgb::new(r, g, b);
    let mut hsv: Hsv = rgb.into_color();

    // 🔹 приглушаем насыщенность
    hsv.saturation *= 0.9;

    // 🔹 слегка затемняем (важно для dark UI)
    hsv.value *= 0.9;

    let rgb2: Srgb = hsv.into_color();

    format!(
        "rgba({}, {}, {}, {})",
        (rgb2.red * 255.0) as u8,
        (rgb2.green * 255.0) as u8,
        (rgb2.blue * 255.0) as u8,
        1.0 // idle transparency (аккуратно!)
    )
}

pub fn luminance(color: &str) -> f32 {
    let (r, g, b, _) = parse_rgba(color)
        .unwrap_or((0.0, 0.0, 0.0, 1.0));

    // защита от двойной нормализации
    let (r, g, b) = if r > 1.0 || g > 1.0 || b > 1.0 {
        (r / 255.0, g / 255.0, b / 255.0)
    } else {
        (r, g, b)
    };

    0.2126 * r + 0.7152 * g + 0.0722 * b
}

pub fn is_light(color: &str) -> bool {
    luminance(color) > 0.6
}

pub fn foreground_color(color: &str, light: String, dark: String) -> String {
    if is_light(color) {
        light
    } else {
        dark    
    }
}

pub fn adjust_brightness(color: &str, factor: f32) -> String {
    let (r, g, b, a) = parse_rgba(color).unwrap_or((0.0, 0.0, 0.0, 1.0));
    let rgb = Srgb::new(r, g, b);
    let mut hsv: Hsv = rgb.into_color();

    hsv.value = (hsv.value * factor).clamp(0.0, 1.0);

    let rgb2: Srgb = hsv.into_color();
    format!(
        "rgba({}, {}, {}, {})",
        (rgb2.red * 255.0) as u8,
        (rgb2.green * 255.0) as u8,
        (rgb2.blue * 255.0) as u8,
        a
    )
}

/// Цвет фона иконки — делаем темнее или светлее в зависимости от яркости базового цвета
pub fn icon_bg_color(color: &str) -> String {
    if is_light(color) {
        // если цвет светлый — затемняем
        adjust_brightness(color, 0.7)
    } else {
        // если тёмный — слегка осветляем
        adjust_brightness(color, 1.3)
    }
}

/// Выбор цвета иконки (на фоне icon_bg_color) — белая или тёмная
pub fn icon_foreground(color: &str) -> String {
    foreground_color(&icon_bg_color(color), "#000000".to_string(), "#FFFFFF".to_string())
}
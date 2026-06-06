use std::io::{Cursor, Read, Write};
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use browser_url::{get_active_browser_url, is_browser_active};
use dioxus::prelude::*;
use base64::Engine;
use image::{ImageBuffer, Rgba};
use windows::Win32::System::Threading::{GetCurrentProcess, GetProcessTimes, GetSystemTimes};
use windows::core::PCWSTR;
use windows::Win32::Foundation::{FILETIME, HWND, RECT};
use windows::Win32::Graphics::Gdi::{
    DeleteObject, GetDC, GetDIBits, GetObjectW, MonitorFromWindow, ReleaseDC, BITMAP, BITMAPINFO,
    BITMAPINFOHEADER, BI_RGB, DIB_RGB_COLORS, HMONITOR, MONITOR_DEFAULTTONEAREST,
};
use windows::Win32::System::ProcessStatus::K32GetModuleFileNameExW;
use windows::Win32::System::SystemInformation::{GetSystemTimeAsFileTime, GetTickCount, GetTickCount64};
use windows::Win32::UI::Input::KeyboardAndMouse::{GetLastInputInfo, LASTINPUTINFO};
use windows::Win32::UI::Shell::ExtractIconExW;
use windows::Win32::UI::WindowsAndMessaging::GetWindowThreadProcessId;
use windows::Win32::UI::WindowsAndMessaging::{
    DestroyIcon, GetClassNameW, GetForegroundWindow, GetIconInfo, GetWindowPlacement,
    GetWindowRect, GetWindowTextLengthW, GetWindowTextW, IsWindowVisible, HICON, ICONINFO,
    SW_MINIMIZE, SW_SHOWMAXIMIZED, WINDOWPLACEMENT,
};
use std::fs::File;

use crate::core::{Rect, WindowBrowser, WindowDesktop, WindowModel, WindowVariant};
use crate::lib::{
    current_ts, get_class_name, get_current_monitor, get_idle_time, get_primary_icon_color, get_window_placement
};

const ICONS_DIR: Asset = asset!("/assets/icons");

pub fn get_current_window(hwnd: Option<HWND>) -> Option<WindowModel> {
    unsafe {
        let hwnd = hwnd.unwrap_or_else(|| GetForegroundWindow());

        if hwnd.0.is_null() {
            return None;
        }

        // STATE
        let placement = get_window_placement(hwnd);

        let is_minimized = placement.showCmd == SW_MINIMIZE.0 as u32;
        let is_maximized = placement.showCmd == SW_SHOWMAXIMIZED.0 as u32;
        let is_visible = IsWindowVisible(hwnd).as_bool();

        // ignore hidden/minimized windows
        if !is_visible || is_minimized {
            return None;
        }

        // TITLE
        let len = GetWindowTextLengthW(hwnd);
        let mut buffer = vec![0u16; (len + 1) as usize];
        GetWindowTextW(hwnd, &mut buffer);

        buffer.truncate(len as usize);
        let title = String::from_utf16_lossy(&buffer);

        // ignore empty titles
        if title.trim().is_empty() {
            return None;
        }

        // RECT
        let mut rect = RECT::default();
        GetWindowRect(hwnd, &mut rect);

        // ignore zero-sized windows
        if rect.right <= rect.left || rect.bottom <= rect.top {
            return None;
        }

        // PID
        let mut pid = 0;
        GetWindowThreadProcessId(hwnd, Some(&mut pid));

        if pid == 0 {
            return None;
        }

        // PROCESS INFO
        let (process_name, process_path) = get_process_info(pid);

        // CLASS NAME
        let class_name = get_class_name(hwnd);

        let monitor = get_current_monitor(hwnd);

        let icon = extract_icon_base64(process_path.as_str());

        let duration = get_idle_time();

        let is_browser = is_browser_active();

        let color = get_primary_icon_color(icon.clone().unwrap_or_default(), process_name.clone());

        Some(WindowModel {
            id: None,
            hwnd: hwnd.0 as isize,
            title,
            class_name,
            process_name,
            process_path,
            variant: match is_browser {
                true => {
                    let info = get_browser_info();
                    match info {
                        Ok((url, browser_name)) => {
                            println!("Browser info: {:?}", url);
                            WindowVariant::Browser(WindowBrowser {
                                browser: browser_name,
                                url,
                            })
                        },
                        Err(err) => WindowVariant::Desktop(WindowDesktop {}),
                    }
                },
                false => WindowVariant::Desktop(WindowDesktop {}),
            },
            pid,
            rect: Rect::from_rect(rect),
            is_minimized,
            is_maximized,
            is_visible,
            is_focused: true,
            monitor_id: Some(monitor.0 as u32),
            timestamp: current_ts(),
            duration,
            icon_base64: icon,
            color,
            tags: Vec::new(),
        })
    }
}

fn get_browser_info() -> Result<(String, String), String> {
    if false && is_browser_active() {
        match get_active_browser_url() {
            Ok(info) => Ok((info.url, info.browser_name)), 
            Err(_) => Err("Failed get browser info".to_string())
        } 
    } else {
        Err("No browser active".to_string())
    }
}

unsafe fn get_process_path(pid: u32) -> String {
    use windows::Win32::System::Threading::{
        OpenProcess, PROCESS_QUERY_INFORMATION, PROCESS_VM_READ,
    };

    let process = OpenProcess(PROCESS_QUERY_INFORMATION | PROCESS_VM_READ, false, pid)
        .expect("Failed to open process");

    if process.is_invalid() {
        return String::from("unknown");
    }

    let mut buffer: Vec<u16> = vec![0; 260];

    let len = K32GetModuleFileNameExW(Some(process), None, &mut buffer);

    if len == 0 {
        return String::from("unknown");
    }

    String::from_utf16_lossy(&buffer[..len as usize])
}

unsafe fn get_process_info(pid: u32) -> (String, String) {
    use windows::Win32::System::ProcessStatus::*;
    use windows::Win32::System::Threading::*;

    let process = OpenProcess(PROCESS_QUERY_INFORMATION | PROCESS_VM_READ, false, pid);

    let process = match process {
        Ok(p) => p,
        Err(_) => return ("unknown".into(), "unknown".into()),
    };

    let mut path = vec![0u16; 260];

    let len = K32GetModuleFileNameExW(Some(process), None, &mut path);

    let process_path = if len > 0 {
        String::from_utf16_lossy(&path[..len as usize])
    } else {
        "unknown".to_string()
    };

    let process_name = process_path
        .split('\\')
        .last()
        .unwrap_or("unknown")
        .to_string();

    (process_name, process_path)
}

unsafe fn extract_icon_base64(path: &str) -> Option<String> {
    let icon_dir = PathBuf::from(ICONS_DIR.bundled().absolute_source_path());
    let icon_name = icon_file_name(path);
    let icon_path = icon_dir.join(&icon_name);

    if icon_path.exists() {
        return load_icon_data_uri(&icon_path);
    }

    let wide: Vec<u16> = path.encode_utf16().chain(Some(0)).collect();

    let mut large_icon = HICON::default();

    let count = ExtractIconExW(PCWSTR(wide.as_ptr()), 0, Some(&mut large_icon), None, 1);

    if count == 0 {
        return load_default_icon_data_uri(&icon_dir);
    }

    let icon = large_icon;

    let mut icon_info = ICONINFO::default();
    GetIconInfo(icon, &mut icon_info);

    let mut bmp = BITMAP::default();
    GetObjectW(
        icon_info.hbmColor.into(),
        std::mem::size_of::<BITMAP>() as i32,
        Some(&mut bmp as *mut _ as *mut _),
    );

    let width = bmp.bmWidth as u32;
    let height = bmp.bmHeight as u32;

    let mut bmi = BITMAPINFO::default();
    bmi.bmiHeader.biSize = std::mem::size_of::<BITMAPINFOHEADER>() as u32;
    bmi.bmiHeader.biWidth = width as i32;
    bmi.bmiHeader.biHeight = -(height as i32);
    bmi.bmiHeader.biPlanes = 1;
    bmi.bmiHeader.biBitCount = 32;
    bmi.bmiHeader.biCompression = BI_RGB.0;

    let mut buffer = vec![0u8; (width * height * 4) as usize];

    let hdc = GetDC(Some(HWND::default()));

    GetDIBits(
        hdc,
        icon_info.hbmColor,
        0,
        height as u32,
        Some(buffer.as_mut_ptr() as *mut _),
        &mut bmi,
        DIB_RGB_COLORS,
    );

    ReleaseDC(Some(HWND::default()), hdc);

    // cleanup
    DeleteObject(icon_info.hbmColor.into());
    DeleteObject(icon_info.hbmMask.into());
    DestroyIcon(icon);

    // BGRA → RGBA
    for chunk in buffer.chunks_exact_mut(4) {
        chunk.swap(0, 2);
    }

    let img: ImageBuffer<Rgba<u8>, _> = ImageBuffer::from_raw(width, height, buffer)?;

    let mut png_bytes = Vec::new();
    img.write_to(
        &mut std::io::Cursor::new(&mut png_bytes),
        image::ImageFormat::Png,
    )
    .ok()?;

    let buff = base64::engine::general_purpose::STANDARD.encode(png_bytes);

    if !icon_dir.exists() {
        let _ = std::fs::create_dir_all(&icon_dir);
    }

    if let Ok(mut file) = File::create(icon_path.clone()) {
        let _ = file.write_all(buff.as_bytes());
    }

    load_icon_data_uri(&icon_path)
}

fn load_icon_data_uri(icon_path: &Path) -> Option<String> {
    if let Ok(mut file) = File::open(icon_path) {
        let mut content = String::new();
        if file.read_to_string(&mut content).is_ok() {
            return Some(format!("data:image/png;base64,{}", content.trim()));
        }
    }
    None
}

fn load_default_icon_data_uri(icon_dir: &PathBuf) -> Option<String> {
    load_icon_data_uri(&icon_dir.join("default.exe.txt"))
}

pub(crate) fn icon_file_name(path: &str) -> String {
    let path = path.trim();
    if path.is_empty() {
        return "default.exe.txt".to_string();
    }

    let binding = PathBuf::from(path);
    let base_name = binding
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("unknown");

    let hash = stable_icon_hash(path);
    format!("{}-{:016x}.txt", sanitize_file_name(base_name), hash)
}

fn sanitize_file_name(value: &str) -> String {
    value
        .chars()
        .filter(|c| c.is_ascii_alphanumeric() || *c == '_' || *c == '-')
        .collect::<String>()
        .to_lowercase()
}

fn stable_icon_hash(value: &str) -> u64 {
    const FNV_OFFSET: u64 = 0xcbf29ce484222325;
    const FNV_PRIME: u64 = 0x100000001b3;
    let mut hash = FNV_OFFSET;
    for byte in value.as_bytes() {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(FNV_PRIME);
    }
    hash
}

pub fn get_uptime() -> u64 {
    unsafe {
        GetTickCount64()
    }
}

const WINDOWS_TO_UNIX_EPOCH_100NS: u64 =
    116444736000000000;

pub fn get_boot_time() -> u64 {
    unsafe {
        let ft = GetSystemTimeAsFileTime();

        let current_100ns =
            ((ft.dwHighDateTime as u64) << 32)
            | ft.dwLowDateTime as u64;

        let uptime_ms = GetTickCount64();

        let boot_100ns =
            current_100ns - uptime_ms * 10_000;

        (boot_100ns - WINDOWS_TO_UNIX_EPOCH_100NS) / 10_000
    }
}

pub fn get_app_uptime() -> u64 {
    unsafe {
        let process = GetCurrentProcess();

        let mut creation = FILETIME::default();
        let mut exit = FILETIME::default();
        let mut kernel = FILETIME::default();
        let mut user = FILETIME::default();

        GetProcessTimes(
            process,
            &mut creation,
            &mut exit,
            &mut kernel,
            &mut user,
        )
        .unwrap();

        let creation_100ns =
            ((creation.dwHighDateTime as u64) << 32)
            | creation.dwLowDateTime as u64;

        let start_ms =
            (creation_100ns - WINDOWS_TO_UNIX_EPOCH_100NS) / 10_000;

        let now_ms = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64;

        now_ms - start_ms
    }
}
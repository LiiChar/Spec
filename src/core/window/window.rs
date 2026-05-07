use std::io::Cursor;

use dioxus::prelude::*;
use base64::Engine;
use image::{ImageBuffer, Rgba};
use windows::core::PCWSTR;
use windows::Win32::Foundation::{HWND, RECT};
use windows::Win32::Graphics::Gdi::{
    DeleteObject, GetDC, GetDIBits, GetObjectW, MonitorFromWindow, ReleaseDC, BITMAP, BITMAPINFO,
    BITMAPINFOHEADER, BI_RGB, DIB_RGB_COLORS, HMONITOR, MONITOR_DEFAULTTONEAREST,
};
use windows::Win32::System::ProcessStatus::K32GetModuleFileNameExW;
use windows::Win32::System::SystemInformation::GetTickCount;
use windows::Win32::UI::Input::KeyboardAndMouse::{GetLastInputInfo, LASTINPUTINFO};
use windows::Win32::UI::Shell::ExtractIconExW;
use windows::Win32::UI::WindowsAndMessaging::GetWindowThreadProcessId;
use windows::Win32::UI::WindowsAndMessaging::{
    DestroyIcon, GetClassNameW, GetForegroundWindow, GetIconInfo, GetWindowPlacement,
    GetWindowRect, GetWindowTextLengthW, GetWindowTextW, IsWindowVisible, HICON, ICONINFO,
    SW_MINIMIZE, SW_SHOWMAXIMIZED, WINDOWPLACEMENT,
};

use crate::core::{Rect, WindowModel};
use crate::lib::{
    current_ts, get_class_name, get_current_monitor, get_idle_time, get_window_placement,
};

const DEFAULT_EXE: Asset = asset!("/assets/exe.png");

pub fn get_current_window(hwnd: Option<HWND>) -> Option<WindowModel> {
    unsafe {
        let hwnd = hwnd.unwrap_or_else(|| GetForegroundWindow());

        if hwnd.0.is_null() {
            return None;
        }

        // TITLE
        let len = GetWindowTextLengthW(hwnd);
        let mut buffer = vec![0u16; (len + 1) as usize];
        GetWindowTextW(hwnd, &mut buffer);

        buffer.truncate(len as usize);
        let title = String::from_utf16_lossy(&buffer);

        // RECT
        let mut rect = RECT::default();
        GetWindowRect(hwnd, &mut rect);

        // STATE
        let placement = get_window_placement(hwnd);

        let is_minimized = placement.showCmd == SW_MINIMIZE.0 as u32;
        let is_maximized = placement.showCmd == SW_SHOWMAXIMIZED.0 as u32;

        let is_visible = IsWindowVisible(hwnd).as_bool();

        // PID
        let mut pid = 0;
        GetWindowThreadProcessId(hwnd, Some(&mut pid));

        // PROCESS INFO
        let (process_name, process_path) = get_process_info(pid);

        // CLASS NAME
        let class_name = get_class_name(hwnd);

        let monitor = get_current_monitor(hwnd);

        let icon = unsafe { extract_icon_base64(process_path.as_str()) };

        let duration = get_idle_time();

        Some(WindowModel {
            hwnd: hwnd.0 as isize,
            title,
            class_name,
            process_name,
            process_path,
            pid,
            rect: Rect::from_rect(rect),
            is_minimized,
            is_maximized,
            is_visible,
            is_focused: true,
            monitor_id: Some(monitor.0 as u32),
            timestamp: current_ts(),
            duration: duration,
            icon_base64: icon,
        })
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
    let wide: Vec<u16> = path.encode_utf16().chain(Some(0)).collect();

    let mut large_icon = HICON::default();

    let count = ExtractIconExW(PCWSTR(wide.as_ptr()), 0, Some(&mut large_icon), None, 1);

    if count == 0 {
        let img: ImageBuffer<Rgba<u8>, Vec<u8>> = image::open(DEFAULT_EXE.bundled().absolute_source_path())
        .expect("Failed to open image")
        .into_rgba8();

        // Create a buffer to hold the PNG data
        let mut bytes: Vec<u8> = Vec::new();
        img.write_to(&mut Cursor::new(&mut bytes), image::ImageFormat::Png)
            .expect("Couldn't write image to bytes.");

        let b64 = base64::engine::general_purpose::STANDARD.encode(bytes);
        Some(format!(
            "data:image/png;base64,{}",
            b64
        ))
    } else {
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

        Some(format!(
            "data:image/png;base64,{}",
            base64::engine::general_purpose::STANDARD.encode(png_bytes)
        ))
    }

}

use windows::Win32::Foundation::{HWND, RECT};
use windows::Win32::Graphics::Gdi::{HMONITOR, MONITOR_DEFAULTTONEAREST, MonitorFromWindow};
use windows::Win32::System::SystemInformation::GetTickCount;
use windows::Win32::UI::Input::KeyboardAndMouse::{GetLastInputInfo, LASTINPUTINFO};
use windows::Win32::UI::WindowsAndMessaging::{
    GetClassNameW, GetForegroundWindow, GetWindowPlacement, GetWindowRect, GetWindowTextLengthW, GetWindowTextW, IsWindowVisible, SW_MINIMIZE, SW_SHOWMAXIMIZED, WINDOWPLACEMENT
};
use windows::Win32::UI::WindowsAndMessaging::GetWindowThreadProcessId;
use windows::Win32::System::ProcessStatus::K32GetModuleFileNameExW;

use crate::core::{Rect, WindowModel};
use crate::lib::{current_ts, get_class_name, get_current_monitor, get_idle_time, get_window_placement};

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
            duration: duration
        })
    }
}

unsafe fn get_process_path(pid: u32) -> String {
    use windows::Win32::System::Threading::{
        OpenProcess, PROCESS_QUERY_INFORMATION, PROCESS_VM_READ,
    };

    let process = OpenProcess(
        PROCESS_QUERY_INFORMATION | PROCESS_VM_READ,
        false,
        pid,
    ).expect("Failed to open process");

    if process.is_invalid() {
        return String::from("unknown");
    }

    let mut buffer: Vec<u16> = vec![0; 260];

    let len = K32GetModuleFileNameExW(
        Some(process),
        None,
        &mut buffer,
    );

    if len == 0 {
        return String::from("unknown");
    }

    String::from_utf16_lossy(&buffer[..len as usize])
}

unsafe fn get_process_info(pid: u32) -> (String, String) {
    use windows::Win32::System::Threading::*;
    use windows::Win32::System::ProcessStatus::*;

    let process = OpenProcess(
        PROCESS_QUERY_INFORMATION | PROCESS_VM_READ,
        false,
        pid,
    );

    let process = match process {
        Ok(p) => p,
        Err(_) => return ("unknown".into(), "unknown".into()),
    };

    let mut path = vec![0u16; 260];

    let len = K32GetModuleFileNameExW(
        Some(process),
        None,
        &mut path,
    );

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


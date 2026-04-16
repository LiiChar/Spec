use windows::Win32::{Foundation::HWND, Graphics::Gdi::{HMONITOR, MONITOR_DEFAULTTONEAREST, MonitorFromWindow}, System::SystemInformation::GetTickCount, UI::{Input::KeyboardAndMouse::{GetLastInputInfo, LASTINPUTINFO}, WindowsAndMessaging::{GetClassNameW, GetWindowPlacement, GetWindowTextLengthW, GetWindowTextW, WINDOWPLACEMENT}}};

pub fn get_class_name(hwnd: HWND) -> String {
    unsafe {
        let mut buf = [0u16; 256];
        
        let len = GetClassNameW(hwnd, &mut buf);
        
        if len == 0 {
            return "unknown".to_string();
        }
        
        String::from_utf16_lossy(&buf[..len as usize])
    }
}

pub fn get_idle_time() -> u64 {
    unsafe {
        let mut info = LASTINPUTINFO {
            cbSize: std::mem::size_of::<LASTINPUTINFO>() as u32,
            dwTime: 0,
        };

        if GetLastInputInfo(&mut info).as_bool() {
            let tick = GetTickCount();
            (tick - info.dwTime) as u64
        } else {
            0
        }
    }
}


pub fn get_window_placement(hwnd: HWND) -> WINDOWPLACEMENT {
    unsafe {
        let mut placement = WINDOWPLACEMENT::default();
        placement.length = std::mem::size_of::<WINDOWPLACEMENT>() as u32;

        match GetWindowPlacement(hwnd, &mut placement) {
            Ok(_) => placement,
            Err(_) => WINDOWPLACEMENT::default(),
        }

    }
}

pub fn get_current_monitor(hwnd: HWND) -> HMONITOR {
    unsafe {
        MonitorFromWindow(hwnd, MONITOR_DEFAULTTONEAREST)
    }
}

pub fn get_window_title(hwnd: HWND) -> String {
    unsafe {
        let len = GetWindowTextLengthW(hwnd);
        if len == 0 {
            return String::new();
        }

        let mut buffer = vec![0u16; (len + 1) as usize];
        GetWindowTextW(hwnd, &mut buffer);

        String::from_utf16_lossy(&buffer[..len as usize])
    }
}
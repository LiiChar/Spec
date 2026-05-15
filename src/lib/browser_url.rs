use std::ffi::OsString;
use std::os::windows::ffi::OsStringExt;
use uiautomation::{UIAutomation, controls::ControlType, types::{UIProperty, PropertyConditionFlags}, variants::Variant};
use windows::Win32::Foundation::{HWND, LPARAM};
use windows::Win32::System::ProcessStatus::K32GetModuleBaseNameW;
use windows::Win32::System::Threading::{OpenProcess, PROCESS_QUERY_INFORMATION, PROCESS_VM_READ};
use windows::Win32::UI::WindowsAndMessaging::{EnumWindows, GetForegroundWindow, GetWindowThreadProcessId, IsIconic, IsWindowVisible};
use windows::core::BOOL;

pub(crate) fn get_foreground_window() -> Result<HWND, String> {
    unsafe {
        let hwnd: HWND = GetForegroundWindow();
        if hwnd.0 == std::ptr::null_mut() {
            return Err("FailedGetForegroundWindow".to_string());
        }
        Ok(hwnd)
    }
}

pub(crate) fn get_visible_windows() -> Result<Vec<HWND>, String> {
    let mut hwnds = Vec::new();
    unsafe {
        if let Err(_) = EnumWindows(Some(enum_windows_callback), LPARAM(&mut hwnds as *mut Vec<HWND> as isize)) {
            return Err("FailedEnumWindow".to_string());
        }
    }
    Ok(hwnds)
}

unsafe extern "system" fn enum_windows_callback(hwnd: HWND, lparam: LPARAM) -> BOOL {
    unsafe {
        let hwnds = &mut *(lparam.0 as *mut Vec<HWND>);
        if IsWindowVisible(hwnd).as_bool() && !IsIconic(hwnd).as_bool() {
            hwnds.push(hwnd);
        }
        BOOL::from(true)
    }
}

pub enum BrowserType {
    Firefox,
    Chrome,
    Edge,
}

pub(crate) fn classify_browser(hwnd: HWND) -> Result<(BrowserType, u32), String> {
    unsafe {
        let mut pid = 0u32;
        GetWindowThreadProcessId(hwnd, Some(&mut pid));
        let process_handle = OpenProcess(PROCESS_QUERY_INFORMATION | PROCESS_VM_READ, false, pid)
            .map_err(|_| "FailedGetForegroundWindow".to_string())?;
        let mut name_buf = vec![0u16; 256];
        let len = K32GetModuleBaseNameW(process_handle, None, &mut name_buf);
        let name = OsString::from_wide(&name_buf[0..len as usize]).to_string_lossy().into_owned();
        if name == "msedge.exe" {
            Ok((BrowserType::Edge, pid))
        } else if name == "chrome.exe" {
            Ok((BrowserType::Chrome, pid))
        } else if name == "firefox.exe" {
            Ok((BrowserType::Firefox, pid))
        } else {
            Err("FailedGetForegroundWindow".to_string())
        }
    }
}

pub(crate) fn get_browser_window_info(only_foreground: bool) -> Result<(BrowserType, u32), String> {
    let visible_windows = if only_foreground {
        vec![get_foreground_window()?]
    } else {
        get_visible_windows()?
    };
    for hwnd in visible_windows {
        return classify_browser(hwnd);
    }
    Err("FailedGetForegroundWindow".to_string())
}

/// Extracts the active tab URL from the specified browser window.
///
/// # Arguments
/// * `timeout` - Maximum time in milliseconds to wait for UI elements to appear (recommended: 3000).
/// * `only_foreground` - If true, searches only the foreground window; otherwise searches all visible windows.
///
/// # Returns
/// * `Ok(String)` - The URL of the active tab.
/// * `Err(String)` - An error message if the URL could not be extracted.
///
/// # Example
/// ```no_run
/// use browser_url_extractor::extract_url;
/// 
/// fn main() {
///     match extract_url(3000, false) {
///         Ok(url) => println!("Current URL: {}", url),
///         Err(e) => eprintln!("Error: {}", e),
///     }
/// }
/// ```

pub fn extract_url(timeout: u64, only_foreground: bool) -> Result<String, String> {
    let (browser_type, pid) = get_browser_window_info(only_foreground)?;
    
    let automation = UIAutomation::new().map_err(|_| "FailedFindUrlUI")?;
    let root = automation.get_root_element().map_err(|_| "FailedFindUrlUI")?;
    
    // No need to create a condition at all – we can use process_id directly
    match browser_type {
        BrowserType::Firefox => {
            let browser = automation
                .create_matcher()
                .from(root)
                .timeout(timeout)
                .process_id(pid)          // ✅ Filters by process ID
                .find_first()
                .map_err(|_| "FailedFindUrlUI")?;
            
            let edit = automation
                .create_matcher()
                .from(browser)
                .timeout(timeout)
                .control_type(ControlType::Edit)
                .find_first()
                .map_err(|_| "FailedFindUrlUI")?;
            
            let url = edit
                .get_property_value(UIProperty::ValueValue)
                .unwrap_or_default()
                .get_string()
                .unwrap_or_default();
            
            if url.is_empty() {
                return Err("FailedExtractUrl".to_string());
            }
            Ok(url)
        }
        BrowserType::Chrome => {
            let browser = automation
                .create_matcher()
                .from(root)
                .timeout(timeout)
                .process_id(pid)          // ✅ Filters by process ID
                .find_first()
                .map_err(|_| "FailedFindUrlUI")?;
            
            let toolbar = automation
                .create_matcher()
                .from(browser)
                .timeout(timeout)
                .classname("ToolbarView")
                .find_first()
                .map_err(|_| "FailedFindUrlUI")?;
            
            let address_bar = automation
                .create_matcher()
                .from(toolbar)
                .timeout(timeout)
                .classname("LocationBarView")
                .find_first()
                .map_err(|_| "FailedFindUrlUI")?;
            
            let edit = automation
                .create_matcher()
                .from(address_bar)
                .timeout(timeout)
                .control_type(ControlType::Edit)
                .find_first()
                .map_err(|_| "FailedFindUrlUI")?;
            
            let url = edit
                .get_property_value(UIProperty::ValueValue)
                .unwrap_or_default()
                .get_string()
                .unwrap_or_default();
            
            if url.is_empty() {
                return Err("FailedExtractUrl".to_string());
            }
            Ok(url)
        }
        BrowserType::Edge => {
            let browser = automation
                .create_matcher()
                .from(root)
                .timeout(timeout)
                .process_id(pid)          // ✅ Filters by process ID
                .find_first()
                .map_err(|_| "FailedFindUrlUI")?;
            
            let toolbar = automation
                .create_matcher()
                .from(browser)
                .timeout(timeout)
                .classname("EdgeToolbarView")
                .find_first()
                .map_err(|_| "FailedFindUrlUI")?;
            
            let address_bar = automation
                .create_matcher()
                .from(toolbar)
                .timeout(timeout)
                .classname("LocationBarView")
                .find_first()
                .map_err(|_| "FailedFindUrlUI")?;
            
            let edit = automation
                .create_matcher()
                .from(address_bar)
                .timeout(timeout)
                .control_type(ControlType::Edit)
                .find_first()
                .map_err(|_| "FailedFindUrlUI")?;
            
            let url = edit
                .get_property_value(UIProperty::ValueValue)
                .unwrap_or_default()
                .get_string()
                .unwrap_or_default();
            
            if url.is_empty() {
                return Err("FailedExtractUrl".to_string());
            }
            Ok(url)
        }
    }
}
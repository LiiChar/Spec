/// Example: Using the refactored database with the new color field

/// Example 1: Getting window color
pub fn example_get_window_color() {
    // In your component or business logic:
    use crate::ui::context::app::use_app_context;
    
    let app_context = use_app_context();
    let db = &app_context.database;
    
    // Get a window's color
    let window_id = 123i64;
    if let Ok(Some(color)) = db.get_window_color(window_id) {
        println!("Window color: {}", color);  // e.g., "bg-blue/20"
    }
}

/// Example 2: Updating window color
pub fn example_update_window_color() {
    use crate::ui::context::app::use_app_context;
    
    let app_context = use_app_context();
    let db = &app_context.database;
    
    let window_id = 123i64;
    let new_color = "bg-red/20";
    
    if let Err(e) = db.update_window_color(window_id, new_color) {
        eprintln!("Failed to update color: {}", e);
    }
}

/// Example 3: Working with color in window operations
pub fn example_windows_with_colors() {
    use crate::ui::context::app::use_app_context;
    
    let app_context = use_app_context();
    let db = &app_context.database;
    
    match db.get_windows() {
        Ok(windows) => {
            for (window, _tags) in windows {
                if let Some(window_id) = window.id {
                    if let Ok(Some(color)) = db.get_window_color(window_id) {
                        println!(
                            "Process: {} | Color: {} | Duration: {}ms",
                            window.process_name,
                            color,
                            window.duration
                        );
                    }
                }
            }
        }
        Err(e) => eprintln!("Error fetching windows: {}", e),
    }
}

/// Example 4: Batch update colors by process
pub fn example_batch_update_colors() {
    use crate::ui::context::app::use_app_context;
    
    let app_context = use_app_context();
    let db = &app_context.database;
    
    // Get all windows for a process
    let process_name = "chrome.exe".to_string();
    if let Ok(windows) = db.get_windows_by_process(process_name) {
        for (window, _tags) in windows {
            if let Some(window_id) = window.id {
                // Color based on some logic
                let color = if window.duration > 3600000 {
                    "bg-red/20"      // Red for > 1 hour
                } else if window.duration > 600000 {
                    "bg-yellow/20"   // Yellow for > 10 min
                } else {
                    "bg-green/20"    // Green otherwise
                };
                
                let _ = db.update_window_color(window_id, color);
            }
        }
    }
}

/// Example 5: Adding color to window on creation
pub fn example_create_window_with_color() {
    use crate::core::WindowModel;
    use crate::ui::context::app::use_app_context;
    
    let app_context = use_app_context();
    let db = &app_context.database;
    
    let mut window = WindowModel {
        id: None,
        hwnd: 0x12345,
        title: "My App".into(),
        class_name: "AppClass".into(),
        icon_base64: None,
        process_name: "myapp.exe".into(),
        process_path: "C:\\Program Files\\myapp\\myapp.exe".into(),
        pid: 1234,
        rect: Default::default(),
        is_minimized: false,
        is_maximized: false,
        is_visible: true,
        is_focused: true,
        monitor_id: Some(1),
        variant: Default::default(),
        timestamp: 0,
        duration: 0,
        color: "bg-blue/20".into(),  // NEW: color field
    };
    
    match db.insert_window(&window) {
        Ok(id) => println!("Window inserted with id: {}", id),
        Err(e) => eprintln!("Failed to insert window: {}", e),
    }
}

/// Example 6: Using schema constants for type safety
pub fn example_using_schema_constants() {
    use crate::core::database::schema::window_activity;
    
    // Instead of error-prone magic strings:
    // let col = "color";  // Wrong - typo possible
    // let sql = format!("SELECT color FROM window_activity ...");
    
    // Use constants:
    let col = window_activity::COL_COLOR;
    let table = window_activity::TABLE;
    let sql = format!(
        "SELECT {} FROM {} WHERE {} = ?1",
        window_activity::COL_COLOR,
        window_activity::TABLE,
        window_activity::COL_ID
    );
    
    println!("SQL: {}", sql);
    // Output: SQL: SELECT color FROM window_activity WHERE id = ?1
}

/// Example 7: Using repositories directly (for advanced use)
pub fn example_repository_usage() {
    use crate::core::database::repositories::WindowRepository;
    use rusqlite::Connection;
    
    fn work_with_repo(conn: &Connection) -> rusqlite::Result<()> {
        // Get all windows
        let windows = WindowRepository::get_all_windows(conn)?;
        
        // Update color for first window
        if let Some((window, _tags)) = windows.first() {
            if let Some(id) = window.id {
                WindowRepository::update_color(conn, id, "bg-purple/20")?;
            }
        }
        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_window_color_field() {
        use crate::core::WindowModel;
        
        let window = WindowModel {
            color: "bg-blue/20".into(),
            ..Default::default()
        };
        
        assert_eq!(window.color, "bg-blue/20");
    }
}

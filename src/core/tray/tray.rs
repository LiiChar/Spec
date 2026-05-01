use std::sync::Arc;

use dioxus::{
    desktop::{
        tao::event::Event,
        trayicon::{
            menu::{Menu, MenuItem},
            Icon, TrayIconBuilder,
        },
        use_tray_menu_event_handler, use_wry_event_handler, WindowEvent,
    },
    prelude::provide_context,
};

pub fn init() {
    let path = concat!("assets/tray_icon.png");
    let icon = load_icon(std::path::Path::new(path));

    let menu = Menu::new();

    let toggle_item = MenuItem::with_id("toggle", "Open", true, None);
    let quit_item = MenuItem::with_id("quit", "Quit", true, None);

    let _ = menu.append_items(&[&toggle_item, &quit_item]).unwrap();

    let toggle_arc = Arc::new(toggle_item);

    let builder = TrayIconBuilder::new()
        .with_menu(Box::new(menu))
        .with_menu_on_left_click(false)
        .with_icon(icon);

    provide_context(builder.build().expect("tray icon builder failed"));

    {
        use_tray_menu_event_handler(move |event| match event.id.0.as_str() {
            "quit" => {
                println!("Quit");
                std::process::exit(0);
            }
            "toggle" => {
                let service = dioxus::desktop::window();
                let window = &service.window;

                let is_visible = window.is_visible();

                window.set_visible(!is_visible);
            }
            _ => {}
        });
    }

    {
        let toggle_arc_clone = toggle_arc.clone();
        use_wry_event_handler(move |event, _| {
            if let Event::WindowEvent {
                window_id: _,
                event,
                ..
            } = event
            {
                match event {
                    WindowEvent::CloseRequested => {
                        toggle_arc_clone.set_text("Open");

                        // Fixing the close behaviour to hide the window fully
                        // By default the app will only hide the webview and keep the window open
                        // Potentially this is something dioxus itself could improve
                        let service = dioxus::desktop::window();
                        let window = &service.window;
                        window.set_visible(false);
                    }
                    _ => {}
                }
            }
        });
    }
}

fn load_icon(path: &std::path::Path) -> Icon {
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

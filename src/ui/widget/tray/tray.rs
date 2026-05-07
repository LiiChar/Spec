use std::fs;
use std::path::Path;

use dioxus::desktop::use_muda_event_handler;
use dioxus::prelude::*;

use dioxus::{
    core::{provide_context, Element},
    desktop::{
        muda::{Menu, MenuItem},
        tao::event::Event,
        trayicon::{Icon, TrayIconBuilder},
        use_wry_event_handler, WindowEvent,
    },
    prelude::rsx,
};

const TRAY_ICON: Asset = asset!("/assets/tray_icon.png");

#[component]
pub fn Tray() -> Element {
    use std::sync::Arc;
    if !Path::new(TRAY_ICON.bundled().absolute_source_path()).exists() {
        return rsx! {
            ""
        };
    }
    let icon = load_icon(&Path::new(TRAY_ICON.bundled().absolute_source_path()));

    let menu = Menu::new();

    let toggle_item = MenuItem::with_id("toggle", "Open", true, None);
    let quit_item = MenuItem::with_id("quit", "Quit", true, None);

    menu.append_items(&[&toggle_item, &quit_item]).unwrap();

    let toggle_arc = Arc::new(toggle_item);

    let builder = TrayIconBuilder::new()
        .with_menu(Box::new(menu))
        .with_menu_on_left_click(false)
        .with_icon(icon)
        .with_title("Spec")
        .with_tooltip("Spec");

    provide_context(builder.build().expect("tray icon builder failed"));

    use_muda_event_handler(move |event| match event.id.0.as_str() {
        "quit" => {
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

    use_wry_event_handler({
        let toggle_arc = toggle_arc.clone();
        move |event, _| {
            if let Event::WindowEvent { event, .. } = event {
                if let WindowEvent::CloseRequested = event {
                    toggle_arc.set_text("Open");

                    let service = dioxus::desktop::window();
                    let window = &service.window;
                    window.set_visible(false);
                }
            }
        }
    });

    rsx! {
     ""
    }
}

fn load_icon(path: &std::path::Path) -> Icon {
    println!("Loading tray icon from: {:?}", path);
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

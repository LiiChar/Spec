use dioxus::prelude::*;

use crate::{core::{TagModel, WindowModel}, ui::components::window::window_item::WindowItem};


#[derive(Props, Clone, PartialEq, Eq)]
pub struct WindowListProps {
    windows: Vec<(WindowModel, Vec<TagModel>)>,
}

#[component]
pub fn WindowList(props: WindowListProps) -> Element {
  rsx! {
    div { class: "flex flex-col gap-1 w-full h-full",

        {
            props
                .windows
                .iter()
                .map(|(window, tags)| {
                    rsx! {
                        WindowItem { window: (window.clone(), tags.clone()) }
                    }
                })
        }
    }
}
}
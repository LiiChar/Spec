use dioxus::prelude::*;
use dioxus_free_icons::icons::ld_icons::{LdTag, LdTrash};
use dioxus_free_icons::Icon;

use crate::{
    core::{with_database, with_database_mut, TagModel, WindowModel},
    ui::{
        use_app, use_toast, Button, Dropdown, DropdownContent, DropdownItem,
        DropdownTrigger,
    },
};

#[derive(Props, Clone, PartialEq, Eq)]
pub struct WindowitemProps {
    pub window: (WindowModel, Vec<TagModel>),
}

#[component]
pub fn WindowItem(props: WindowitemProps) -> Element {
    let mut all_tags = use_app().tags;

    let app = use_app();
    let mut toast = use_toast();

    let (window, tags) = &props.window;

    let process = window.process_name.clone();
    let process_f = window.process_name.clone();
    let process_s = window.process_name.clone();

    let title = if window.title.trim().is_empty() {
        window.process_name.as_str()
    } else {
        window.title.as_str()
    };

    let refresh = Callback::new(move |_: ()| {
        app.refresh_events();
    });

    let info = Callback::new(move |title: String| {
        toast.info(title, None, None);
    });

    rsx! {
        div { class: "flex justify-between items-center gap-3 p-2 rounded-md hover:bg-secondary/30 transition-colors",

            div { class: "flex items-center gap-2 min-w-0 flex-1",

                if let Some(icon) = window.icon_base64.as_deref() {
                    img { class: "w-5 h-5 rounded shrink-0", src: icon }
                } else {
                    div { class: "w-5 h-5 rounded bg-secondary shrink-0" }
                }

                div { class: "min-w-0 flex flex-col",

                    span { class: "text-xs font-medium text-foreground truncate", "{title}" }

                    span { class: "text-[10px] text-muted-foreground truncate", "{window.process_name}" }
                }
            }

            div {
                div { class: "flex gap-1",

                    Button {
                        onclick: move |_| {
                            let process = process_f.clone();

                            spawn(async move {
                                let result = with_database_mut(|db| {
                                    db.delete_window(process.clone().to_string())
                                });

                                match result {
                                    Ok(_) => {
                                        refresh(());
                                        info(
                                            format!(
                                                "Удалено приложение {}",
                                                process.clone(),
                                            ),
                                        );
                                    }
                                    Err(_) => {}
                                }
                            });
                        },

                        Icon { width: 12, height: 12, icon: LdTrash }
                    }

                    Dropdown {
                        DropdownTrigger {
                            Button {
                                Icon { width: 12, height: 12, icon: LdTag }
                            }
                        }

                        DropdownContent {
                            {
                                let value = process_s.clone();
                                all_tags
                                    .read()
                                    .clone()
                                    .into_iter()
                                    .map(move |tag| {
                                        let process = value.clone();
                                        let tag_id = tag.clone().id.unwrap();
                                        let tag_name = tag.clone().name.clone();
                                        let color = tag.clone().color;
                                        rsx! {
                                            DropdownItem {
                                                onclick: move |_| {
                                                    let process = process.clone();
                                                    let tag_name_s = tag.clone().name.clone();

                                                    spawn(async move {
                                                        let result = with_database_mut(|db| {
                                                            db.add_tag_to_window(tag_id, process.clone().to_string())
                                                        });

                                                        match result {
                                                            Ok(_) => {
                                                                refresh(());
                                                                info(format!("Добавлен тег {}", tag_name_s));
                                                            }
                                                            Err(_) => {}
                                                        }
                                                    });
                                                },
                                                span {
                                                    class: "text-[10px] px-1.5 py-0.5 rounded border",
                                                    style: format!("border-color:{}; color:{};", color, color),
                                                    "{tag_name}"
                                                }
                                            }
                                        }
                                    })
                            }
                        }
                    }
                }

                if !tags.is_empty() {
                    div { class: "flex gap-1 shrink-0 flex-wrap justify-end",

                        for tag in tags.iter().take(3) {
                            span {
                                class: "text-[10px] px-1.5 py-0.5 rounded border",
                                style: format!("border-color:{}; color:{};", tag.color, tag.color),
                                "{tag.name}"
                            }
                        }

                        if tags.len() > 3 {
                            span { class: "text-[10px] text-muted-foreground", "+{tags.len() - 3}" }
                        }
                    }
                }
            }
        }
    }
}
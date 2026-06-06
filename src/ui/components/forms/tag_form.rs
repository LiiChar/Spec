use dioxus::prelude::*;

use crate::{
    core::TagModel,
    ui::ColorPicker,
};

#[derive(Props, PartialEq, Clone)]
pub struct TagFormProps {
    #[props(default = None)]
    pub tag: Option<TagModel>,

    pub on_save: Callback<TagModel>,
    pub on_cancel: Callback<()>,
}

#[component]
pub fn TagForm(props: TagFormProps) -> Element {
    let is_edit = props.tag.is_some();

    let mut name = use_signal(|| {
        props
            .tag
            .as_ref()
            .map(|t| t.name.clone())
            .unwrap_or_default()
    });

    let mut description = use_signal(|| {
        props
            .tag
            .as_ref()
            .and_then(|t| t.description.clone())
            .unwrap_or_default()
    });

    let mut color = use_signal(|| {
        props
            .tag
            .as_ref()
            .map(|t| t.color.clone())
            .unwrap_or("#3b82f6".to_string())
    });

    let mut filter = use_signal(|| {
        props
            .tag
            .as_ref()
            .and_then(|t| t.filter.clone())
            .unwrap_or_default()
    });

    rsx! {
        h2 {
            class: "text-lg font-semibold mb-4",

            if is_edit {
                "Редактирование тега"
            } else {
                "Создание тега"
            }
        }

        div {
            class: "space-y-4",

            div {
                label {
                    class: "text-sm font-medium",
                    "Название*"
                }

                input {
                    class: "w-full px-3 py-2 border rounded-md bg-background",

                    value: "{name}",

                    oninput: move |e| {
                        name.set(e.value());
                    },

                    placeholder: "Работа"
                }
            }

            div {
                label {
                    class: "text-sm font-medium",
                    "Описание"
                }

                textarea {
                    class: "w-full px-3 py-2 border rounded-md bg-background min-h-[100px]",

                    value: "{description}",

                    oninput: move |e| {
                        description.set(e.value());
                    },

                    placeholder: "Описание тега"
                }
            }

            div {
                label {
                    class: "text-sm font-medium",
                    "Regex фильтр"
                }

                input {
                    class: "w-full px-3 py-2 border rounded-md bg-background",

                    value: "{filter}",

                    oninput: move |e| {
                        filter.set(e.value());
                    },

                    placeholder: "chrome|firefox|edge"
                }

                p {
                    class: "text-xs text-muted-foreground mt-1",
                    "Используется для автоматического назначения тега."
                }
            }

            div {
                label {
                    class: "text-sm font-medium block mb-2",
                    "Цвет"
                }

                ColorPicker {
                    color,

                    onselect: move |c: String| {
                        color.set(c);
                    }
                }
            }
        }

        div {
            class: "flex justify-end gap-2 mt-6",

            button {
                class: "px-4 py-2 border rounded-md",

                onclick: move |_| {
                    props.on_cancel.call(());
                },

                "Отмена"
            }

            button {
                class: "px-4 py-2 bg-primary text-white rounded-md",

                onclick: move |_| {
                    if name().trim().is_empty() {
                        return;
                    }

                    let mut tag = props
                        .tag
                        .clone()
                        .unwrap_or_else(|| {
                            TagModel::new(
                                name().trim(),
                                None,
                                color(),
                                None,
                            )
                        });

                    tag.name = name().trim().to_string();

                    tag.description = if description().trim().is_empty() {
                        None
                    } else {
                        Some(description().trim().to_string())
                    };

                    tag.color = color();

                    tag.filter = if filter().trim().is_empty() {
                        None
                    } else {
                        Some(filter().trim().to_string())
                    };

                    props.on_save.call(tag);
                },

                if is_edit {
                    "Сохранить"
                } else {
                    "Создать"
                }
            }
        }
    }
}
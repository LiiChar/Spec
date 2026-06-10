use dioxus::prelude::*;
use dioxus_free_icons::icons::ld_icons::{LdX, LdPencil, LdTrash};
use dioxus_free_icons::Icon;

use crate::{
    core::TagModel,
    ui::{button::Button, use_alert, use_toast},
    ui::tag_form::TagForm,
    ui::use_app,
};

#[derive(Props, PartialEq, Clone)]
pub struct TagModalProps {
    pub tag: Option<TagModel>,
    #[props(default = Callback::new(|_| ()))]
    pub on_close: Callback<()>,
    #[props(default = Signal::new(false))]
    pub visible: WriteSignal<bool>,
}

#[component]
pub fn TagModal(props: TagModalProps) -> Element {
    if !(*props.visible.read()) {
        return rsx! { "" };
    }

    let app = use_app();
    let mut toast = use_toast();
    let mut alert = use_alert();
    let mut visible_update = use_signal(|| false);

    let tag = match props.tag.clone() {
        Some(t) => t,
        None => return rsx! { "" },
    };

    let mut update = {
        let app = app.clone();
        let mut toast = toast.clone();
        move |tag: TagModel| {
            app.update_tag(tag.clone());
            toast.info("Тег обновлён".to_string(), None, None);
        }
    };

    let delete = {
        let app = app.clone();
        let mut toast = toast.clone();
        let tag_id = tag.id;

        Callback::new(move |_: ()| {
            let app = app.clone();
            let mut toast = toast.clone();
            let tag_id = tag_id;

            alert.error(
                "Удалить тег?".to_string(),
                None,
                None,
                Callback::new(move |ok| {
                    if ok {
                        if let Some(id) = tag_id {
                            app.delete_tag(id);
                            toast.info("Тег удалён".to_string(), None, None);
                        }
                        props.on_close.call(());
                    }
                }),
            );
        })
    };

    rsx! {
        div {
            class: "fixed inset-0 bg-black/50 flex items-center justify-center p-4 z-[200]",
            onclick: move |_| props.on_close.call(()),

            div {
                class: "bg-background p-6 rounded-lg shadow-lg max-w-md w-full max-h-[80vh] overflow-auto relative",
                style: format!("border-bottom: 2px solid {}", tag.color),
                onclick: move |evt| evt.stop_propagation(),

                // кнопка закрытия
                button {
                    class: "absolute top-2 right-2 hover:bg-destructive rounded-lg p-1 transition-colors",
                    onclick: move |_| props.on_close.call(()),
                    Icon { icon: LdX }
                }

                // отображение информации о теге
                if !visible_update() {
                    div {
                        class: "space-y-3",

                        h2 { class: "text-xl font-semibold", "{tag.name}" }

                        if let Some(desc) = &tag.description {
                            p { class: "text-sm text-muted-foreground", "{desc}" }
                        }

                        if let Some(filter) = &tag.filter {
                            div { class: "text-xs opacity-70 mt-2", "Regex: {filter}" }
                        }

                        div {
                            class: "mt-4 flex gap-2 justify-end",

                            Button {
                                onclick: move |_| delete(()),
                                Icon { icon: LdTrash }
                            }

                            Button {
                                onclick: move |_| visible_update.set(true),
                                Icon { icon: LdPencil }
                            }
                        }
                    }
                }

                // форма редактирования
                if visible_update() {
                    TagForm {
                        tag: Some(tag.clone()),

                        on_save: move |updated: TagModel| {
                            update(updated);
                            visible_update.set(false);
                        },

                        on_cancel: move |_| {
                            visible_update.set(false);
                        }
                    }
                }
            }
        }
    }
}
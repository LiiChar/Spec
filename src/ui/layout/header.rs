use dioxus::{desktop::use_window, prelude::*};

#[component]
pub fn Header() -> Element {
      let window = use_window();

    let drag_window = window.clone();
    let close_window = window.clone();

  rsx! {
    div {
      class: "fixed top-0 left-0 w-full flex",
      div {
        onmousedown: move |_| {
          drag_window.drag();
        },
        class: "w-full h-[10px] "
      }
      button {
        z_index: 10,
        onclick: move |e: MouseEvent| {
          e.stop_propagation();
          close_window.close();
        },
        "X"
      }
    }
  }
}
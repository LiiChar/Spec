use dioxus::prelude::*;

#[derive(Props, Clone, PartialEq)]
pub struct TimeInputProps {
    pub value: WriteSignal<u32>,
}

#[component]
pub fn TimeInput(props: TimeInputProps) -> Element {
    let mut value = props.value;

    let total_seconds = *value.read();

    let hours = total_seconds / 3600;
    let minutes = (total_seconds % 3600) / 60;
    let seconds = total_seconds % 60;

    let normalize = |h: i32, m: i32, s: i32| -> u32 {
        let mut seconds = s;
        let mut minutes = m;
        let mut hours = h;

        if seconds >= 60 {
            minutes += seconds / 60;
            seconds %= 60;
        } else if seconds < 0 {
            let borrow = (-seconds + 59) / 60;
            seconds += borrow * 60;
            minutes -= borrow;
        }

        if minutes >= 60 {
            hours += minutes / 60;
            minutes %= 60;
        } else if minutes < 0 {
            let borrow = (-minutes + 59) / 60;
            minutes += borrow * 60;
            hours -= borrow;
        }

        if hours < 0 {
            return 0;
        }

        let total: i64 = hours as i64 * 3600 + minutes as i64 * 60 + seconds as i64;
        if total < 0 {
            0
        } else {
            total as u32
        }
    };

    rsx! {
        div { class: "flex justofy-center gap-0.5 h-full w-full",

            // HOURS
            input {
                class: "w-[calc(100%/3-6px)] text-center",
                r#type: "number",
                max: "24",
                min: "0",
                value: if hours > 9 { format!("{hours}") } else { format!("0{hours}") },
                oninput: move |evt| {
                    if let Ok(h) = evt.value().parse::<i32>() {
                        let current = *value.read();
                        let m = (current % 3600) / 60;
                        let s = current % 60;

                        value.set(normalize(h, m as i32, s as i32));
                    }
                },
            }
            span { ":" }
            // MINUTES
            input {
                class: "w-[calc(100%/3-6px)] text-center",
                r#type: "number",
                value: if minutes > 9 { format!("{minutes}") } else { format!("0{minutes}") },
                oninput: move |evt| {
                    if let Ok(m) = evt.value().parse::<i32>() {
                        let current = *value.read();
                        let h = current / 3600;
                        let s = current % 60;

                        value.set(normalize(h as i32, m, s as i32));
                    }
                },
            }
            span { ":" }
            // SECONDS
            input {
                class: "w-[calc(100%/3-6px)] text-center",
                r#type: "number",
                value: if seconds > 9 { format!("{seconds}") } else { format!("0{seconds}") },
                oninput: move |evt| {
                    if let Ok(s) = evt.value().parse::<i32>() {
                        let current = *value.read();
                        let h = current / 3600;
                        let m = (current % 3600) / 60;

                        value.set(normalize(h as i32, m as i32, s));
                    }
                },
            }
        }
    }
}

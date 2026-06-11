use std::collections::{BTreeMap, HashMap};

use chrono::{Datelike, NaiveDate, Timelike};
use dioxus::prelude::*;
use dioxus_free_icons::icons::ld_icons::LdTrash;
use dioxus_free_icons::Icon;

use crate::{
    core::{EventModel, EventType, GoalModel, JobModel, get_app_uptime, get_boot_time, get_uptime, with_database, with_database_mut},
    lib::{convert_ts_to_local_date, format_duration, format_duration_short}, ui::{components::{forms::{goal_form::GoalForm, job_form::JobForm}, modal::tag_modal::TagModal, ui::{button::Button, calendar::Calendar, tooltip::{Tooltip, TooltipAlign}}}, context::{use_app, use_toast}, widget::window::window::Windows},
};
use chrono::Local;

#[derive(Debug, Clone, PartialEq)]
struct AppUsage {
    name: String,
    active_ms: u64,
    idle_ms: u64,
    events: usize,
    days: usize,
    icon: Option<String>,
}

impl AppUsage {
    fn total_ms(&self) -> u64 {
        self.active_ms + self.idle_ms
    }
}

#[derive(Debug, Clone, PartialEq)]
struct DayUsage {
    date: NaiveDate,
    total_ms: u64,
    active_ms: u64,
    idle_ms: u64,
    events: usize,
    top_app: String,
}

#[derive(Debug, Clone, PartialEq)]
struct HourUsage {
    hour: u32,
    total_ms: u64,
    active_ms: u64,
}

#[derive(Debug, Clone, PartialEq)]
struct StatsView {
    events: Vec<EventModel>,
    apps: Vec<AppUsage>,
    days: Vec<DayUsage>,
    hours: Vec<HourUsage>,
    total_ms: u64,
    active_ms: u64,
    idle_ms: u64,
    avg_day_ms: u64,
    avg_event_ms: u64,
    peak_day: Option<DayUsage>,
    peak_hour: Option<HourUsage>,
    longest_event: Option<EventModel>,
}

fn parse_date(value: &str) -> Option<NaiveDate> {
    NaiveDate::parse_from_str(value, "%Y-%m-%d").ok()
}

fn job_anchor_day(job: &JobModel) -> NaiveDate {
    convert_ts_to_local_date((job.start_ts as i64 * 1000) as u64).date_naive()
}

fn event_app(event: &EventModel) -> String {
    event
        .window
        .as_ref()
        .map(|window| window.process_name.clone())
        .unwrap_or_else(|| "Unknown".to_string())
}

fn analyze_events(
    events: &[EventModel],
    date_from: &str,
    date_to: &str,
    app_filter: &str,
) -> StatsView {
    let from = parse_date(date_from);
    let to = parse_date(date_to);
    let app_filter = app_filter.to_lowercase();

    let mut filtered = Vec::new();
    for event in events {
        let date = convert_ts_to_local_date(event.timestamp).date_naive();
        if from.is_some_and(|from| date < from) || to.is_some_and(|to| date > to) {
            continue;
        }

        let app = event_app(event);
        if !app_filter.is_empty() && !app.to_lowercase().contains(&app_filter) {
            continue;
        }

        filtered.push(event.clone());
    }

    let total_ms = filtered.iter().map(|event| event.duration).sum::<u64>();
    let active_ms = filtered
        .iter()
        .filter(|event| event.event_type != EventType::Idle)
        .map(|event| event.duration)
        .sum::<u64>();
    let idle_ms = total_ms.saturating_sub(active_ms);
    let avg_event_ms = if filtered.is_empty() {
        0
    } else {
        total_ms / filtered.len() as u64
    };

    let mut app_map: HashMap<String, AppUsage> = HashMap::new();
    let mut app_days: HashMap<String, Vec<NaiveDate>> = HashMap::new();
    let mut day_map: BTreeMap<NaiveDate, (u64, u64, u64, usize, HashMap<String, u64>)> =
        BTreeMap::new();
    let mut hour_map: BTreeMap<u32, (u64, u64)> = BTreeMap::new();

    for event in &filtered {
        let date_time = convert_ts_to_local_date(event.timestamp);
        let date = date_time.date_naive();
        let hour = date_time.hour();
        let app = event_app(event);
        let is_idle = event.event_type == EventType::Idle;

        let entry = app_map.entry(app.clone()).or_insert_with(|| AppUsage {
            name: app.clone(),
            active_ms: 0,
            idle_ms: 0,
            events: 0,
            days: 0,
            icon: event
                .window
                .as_ref()
                .and_then(|window| window.icon_base64.clone()),
        });
        if is_idle {
            entry.idle_ms += event.duration;
        } else {
            entry.active_ms += event.duration;
        }
        entry.events += 1;
        app_days.entry(app.clone()).or_default().push(date);

        let day_entry = day_map
            .entry(date)
            .or_insert_with(|| (0, 0, 0, 0, HashMap::new()));
        day_entry.0 += event.duration;
        if is_idle {
            day_entry.2 += event.duration;
        } else {
            day_entry.1 += event.duration;
        }
        day_entry.3 += 1;
        *day_entry.4.entry(app).or_default() += event.duration;

        let hour_entry = hour_map.entry(hour).or_insert((0, 0));
        hour_entry.0 += event.duration;
        if !is_idle {
            hour_entry.1 += event.duration;
        }
    }

    let mut apps = app_map.into_values().collect::<Vec<_>>();
    for app in &mut apps {
        if let Some(days) = app_days.get_mut(&app.name) {
            days.sort();
            days.dedup();
            app.days = days.len();
        }
    }
    apps.sort_by(|a, b| b.total_ms().cmp(&a.total_ms()));

    let mut days = Vec::new();
    for (date, (total, active, idle, events, apps_by_time)) in day_map {
        let top_app = apps_by_time
            .into_iter()
            .max_by_key(|(_, time)| *time)
            .map(|(app, _)| app)
            .unwrap_or_else(|| "Unknown".to_string());
        days.push(DayUsage {
            date,
            total_ms: total,
            active_ms: active,
            idle_ms: idle,
            events,
            top_app,
        });
    }

    let hours = (0..24)
        .map(|hour| {
            let (total, active) = hour_map.get(&hour).copied().unwrap_or_default();
            HourUsage {
                hour,
                total_ms: total,
                active_ms: active,
            }
        })
        .collect::<Vec<_>>();

    let avg_day_ms = if days.is_empty() {
        0
    } else {
        total_ms / days.len() as u64
    };
    let peak_day = days.iter().max_by_key(|day| day.total_ms).cloned();
    let peak_hour = hours.iter().max_by_key(|hour| hour.total_ms).cloned();
    let longest_event = filtered.iter().max_by_key(|event| event.duration).cloned();

    StatsView {
        events: filtered,
        apps,
        days,
        hours,
        total_ms,
        active_ms,
        idle_ms,
        avg_day_ms,
        avg_event_ms,
        peak_day,
        peak_hour,
        longest_event,
    }
}

#[component]
pub fn StatisticsPage() -> Element {
    let mut all_events = use_signal(Vec::<EventModel>::new);
    let mut is_loading = use_signal(|| true);
    let mut load_error = use_signal(String::new);
    let mut date_from = use_signal(String::new);
    let mut date_to = use_signal(String::new);
    let mut app_filter = use_signal(String::new);

    let mut tag_modal_visible = use_signal(|| false);

    use_effect(move || {
        spawn(async move {
            let result = tokio::task::spawn_blocking(move || {
                with_database(|db| db.get_all_events().map_err(|err| err.to_string()))
            })
            .await;

            match result {
                Ok(Ok(events)) => {
                    all_events.set(events);
                    load_error.set(String::new());
                }
                Ok(Err(err)) => load_error.set(err),
                Err(err) => load_error.set(err.to_string()),
            }
            is_loading.set(false);
        });
    });

    let stats =
        use_memo(move || analyze_events(&all_events(), &date_from(), &date_to(), &app_filter()));
    let view = stats();
    let most_used_app = view.apps.first().cloned();
    let active_percent = if view.total_ms > 0 {
        (view.active_ms as f64 / view.total_ms as f64) * 100.0
    } else {
        0.0
    };

    rsx! {
        div { class: "mx-auto flex w-full max-w-7xl flex-col gap-4 p-2 pb-20",
            TagModal {
                visible: tag_modal_visible.clone(),
                on_close: move |_| tag_modal_visible.set(false),
            }
            div { class: "flex flex-col gap-1",
                h1 { class: "text-xl font-semibold text-foreground",
                    "Статистика использования"
                }
            }

            section { class: "rounded-md border border-border/40 bg-background/70 p-4",
                div { class: "grid gap-3 md:grid-cols-4",
                    label { class: "flex flex-col gap-2 text-sm text-foreground/70",
                        "Дата от"
                        Calendar { onselect: move |date: NaiveDate| date_from.set(date.format("%Y-%m-%d").to_string()) }
                    }
                    label { class: "flex flex-col gap-2 text-sm text-foreground/70",
                        "Дата до"
                        Calendar { onselect: move |date: NaiveDate| date_to.set(date.format("%Y-%m-%d").to_string()) }
                    }
                    label { class: "flex flex-col gap-2 text-sm text-foreground/70 md:col-span-2",
                        "Приложение"
                        input {
                            class: "h-10 rounded-md border border-border/40 bg-background px-3 text-foreground outline-none focus:border-primary",
                            placeholder: "chrome, code, telegram...",
                            value: "{app_filter()}",
                            oninput: move |evt| app_filter.set(evt.value()),
                        }
                    }
                }
            }

            GoalsJobsPanel {}

            

            if is_loading() {
                div { class: "rounded-md border border-border/40 bg-background/70 p-4 text-sm text-foreground/70",
                    "Загружаю статистику..."
                }
            } else if !load_error().is_empty() {
                div { class: "rounded-md border border-red-500/40 bg-red-500/10 p-4 text-sm text-red-400",
                    "{load_error}"
                }
            } else {
                div { class: "grid gap-3 grid-cols-2 md:grid-cols-3 xl:grid-cols-4",
                    MetricCard {
                        title: "Общее время".to_string(),
                        value: format_duration_short(view.total_ms),
                        hint: format!("{} событий", view.events.len()),
                    }
                    MetricCard {
                        title: "Активное время".to_string(),
                        value: format_duration_short(view.active_ms),
                        hint: format!("{active_percent:.0}% от общего"),
                    }
                    MetricCard {
                        title: "Средний день".to_string(),
                        value: format_duration_short(view.avg_day_ms),
                        hint: format!("{} дней с событиями", view.days.len()),
                    }
                    MetricCard {
                        title: "Популярное приложение".to_string(),
                        value: most_used_app
                            .as_ref()
                            .map(|app| app.name.clone())
                            .unwrap_or_else(|| "Нет данных".to_string()),
                        hint: most_used_app
                            .as_ref()
                            .map(|app| format_duration_short(app.total_ms()))
                            .unwrap_or_default(),
                    }
                    MetricCard {
                        title: "Время работы приложения".to_string(),
                        value: format_duration_short(get_app_uptime()),
                        hint: format!(""),
                    }
                    MetricCard {
                        title: "Общее работы компльютера".to_string(),
                        value: format_duration_short(get_uptime()),
                        hint: format!(""),
                    }
                    MetricCard {
                        title: "Время запуска компьютера".to_string(),
                        value: convert_ts_to_local_date(get_boot_time() as u64).format("%H:%M:%S").to_string(),
                        hint: format!(""),
                    }
                }

                div { class: "grid gap-4 xl:grid-cols-[1.2fr_0.8fr]",
                    section { class: "rounded-md border border-border/40 bg-background/70 p-4",
                        div { class: "mb-4 flex items-center justify-between gap-3",
                            h2 { class: "text-base font-semibold text-foreground",
                                "Ежедневное использование"
                            }
                            if let Some(day) = view.peak_day.clone() {
                                span { class: "text-xs text-foreground/55",
                                    "Пиковый день: {day.date.format(\"%d.%m.%Y\")} · {format_duration_short(day.total_ms)}"
                                }
                            }
                        }
                        DailyUsageChart { days: view.days.clone() }
                    }

                    section { class: "rounded-md border border-border/40 bg-background/70 p-4",
                        h2 { class: "mb-4 text-base font-semibold text-foreground",
                            "Ключевые метрики"
                        }
                        div { class: "grid gap-3",
                            InfoRow {
                                label: "Приложений".to_string(),
                                value: view.apps.len().to_string(),
                            }
                            InfoRow {
                                label: "Idle время".to_string(),
                                value: format_duration_short(view.idle_ms),
                            }
                            InfoRow {
                                label: "Среднее событие".to_string(),
                                value: format_duration_short(view.avg_event_ms),
                            }
                            InfoRow {
                                label: "Пиковый час".to_string(),
                                value: view.peak_hour
                                    .as_ref()
                                    .map(|hour| {
                                        format!("{:02}:00 · {}", hour.hour, format_duration_short(hour.total_ms))
                                    })
                                    .unwrap_or_else(|| "Нет данных".to_string()),
                            }
                        }
                    }
                }

                section { class: "rounded-md border border-border/40 bg-background/70 p-4",
                    h2 { class: "mb-4 text-base font-semibold text-foreground",
                        "Активность по часам"
                    }
                    HourActivityChart { hours: view.hours.clone() }
                }

                div { class: "grid gap-4 xl:grid-cols-2",
                    section { class: "rounded-md border border-border/40 bg-background/70 p-4",
                        div { class: "mb-4 flex flex-wrap items-center justify-between gap-3",
                            div { class: "flex justify-between items-center gap-3 w-full",
                                h2 { class: "text-base font-semibold text-foreground", "Приложения" }
                                div {
                                    Button {
                                        class: "px-3! py-1.5! text-xs font-medium",
                                        onclick: move |_| {
                                            tag_modal_visible.set(true);
                                        },
                                        "Добавить тег"
                                    }
                                }
                            }
                        }

                        div { class: "flex flex-wrap gap-2 max-h-96 overflow-y-auto", Windows {} }
                    }

                    section { class: "rounded-md border border-border/40 bg-background/70 p-4",
                        h2 { class: "mb-4 text-base font-semibold text-foreground",
                            "Дни подробно"
                        }
                        div { class: "flex max-h-[520px] flex-col gap-2 overflow-auto pr-1",
                            for day in view.days.iter().rev().cloned() {
                                DayRow { day }
                            }
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn GoalsJobsPanel() -> Element {
    let app = use_app();
    let mut toast = use_toast();

    let u_app= app.clone();

    let update = move |job: JobModel| {
        u_app.update_jobs(job);
    };

    let jobs = app.jobs;
    let goals = app.goals;
    let mut show_goal_form = use_signal(|| false);
    let mut goal_edit = use_signal(|| None::<GoalModel>);
    let mut show_job_form = use_signal(|| false);
    let mut job_edit = use_signal(|| None::<JobModel>);

    let goals_done = goals().iter().filter(|g| g.completed).count();
    let jobs_tagged = jobs().iter().filter(|j| !j.tags.is_empty()).count();
    let day = Local::now().date_naive();

    let d_app = app.clone();
    let delete = Callback::new(move |id: i64| { 
        d_app.delete_job(id);
    });


    let info = Callback::new(move |title: String| {
        toast.info(title, None, None);
    });

    let jobs_list = jobs()
    .clone()
    .into_iter()
    .take(40)
    .map(|j| {
        let edit_job = j.clone();
        let delete_id = j.id.unwrap();
        (j, edit_job, delete_id)
    })
    .collect::<Vec<_>>();

    rsx! {
        section { class: "rounded-md border border-border/40 bg-background/70 p-4",
            div { class: "mb-3 flex flex-wrap items-center justify-between gap-3",
                div {
                    h2 { class: "text-base font-semibold text-foreground", "Задачи" }
                    p { class: "text-xs text-foreground/55",
                        "Сводка по записям в базе: задачи с тегами и расписанием."
                    }
                }
                div { class: "flex flex-wrap gap-2",
                    // button {
                    //     class: "rounded-md border border-primary/40 bg-primary/15 px-3 py-1.5 text-xs font-medium text-foreground hover:bg-primary/25",
                    //     onclick: move |_| {
                    //         goal_edit.set(None);
                    //         show_goal_form.set(true);
                    //     },
                    //     "Новая цель"
                    // }
                    button {
                        class: "rounded-md border border-border/40 bg-background px-3 py-1.5 text-xs font-medium text-foreground hover:bg-foreground/5",
                        onclick: move |_| {
                            job_edit.set(None);
                            show_job_form.set(true);
                        },
                        "Новая задача"
                    }
                }
            }

            div { class: "grid gap-3 sm:grid-cols-2 lg:grid-cols-4",
                // div { class: "rounded-md border border-border/30 bg-foreground/5 p-3",
                //     div { class: "text-xs text-foreground/55", "Целей всего" }
                //     div { class: "text-lg font-semibold text-foreground", "{goals().len()}" }
                // }
                // div { class: "rounded-md border border-border/30 bg-foreground/5 p-3",
                //     div { class: "text-xs text-foreground/55", "Целей выполнено" }
                //     div { class: "text-lg font-semibold text-foreground", "{goals_done}" }
                // }
                div { class: "rounded-md border border-border/30 bg-foreground/5 p-3",
                    div { class: "text-xs text-foreground/55", "Задач всего" }
                    div { class: "text-lg font-semibold text-foreground", "{jobs().len()}" }
                }
                div { class: "rounded-md border border-border/30 bg-foreground/5 p-3",
                    div { class: "text-xs text-foreground/55", "Задач с тегами" }
                    div { class: "text-lg font-semibold text-foreground", "{jobs_tagged}" }
                }
            }

            div { class: "mt-4 grid gap-4 lg:grid-cols-2",
                // div {
                //     h3 { class: "mb-2 text-sm font-medium text-foreground", "Цели" }
                //     div { class: "flex max-h-64 flex-col gap-2 overflow-auto pr-1",
                //         if goals().is_empty() {
                //             div { class: "text-sm text-foreground/55", "Пока нет целей — создайте первую." }
                //         }
                //         for g in goals().iter().cloned() {
                //             div { class: "rounded-md border border-border/30 p-3",
                //                 div { class: "flex items-start justify-between gap-2",
                //                     div {
                //                         div { class: "text-sm font-medium text-foreground", "{g.name}" }
                //                         div { class: "text-xs text-foreground/55", "{g.process}" }
                //                         if let Some(desc) = g.description.clone() {
                //                             div { class: "mt-1 text-xs text-foreground/60", "{desc}" }
                //                         }
                //                         div { class: "mt-2 flex flex-wrap gap-1",
                //                             for t in g.tags.iter().cloned() {
                //                                 span { class: "rounded-full border border-border/40 px-2 py-0.5 text-[10px] text-foreground/80",
                //                                     "{t.name}"
                //                                 }
                //                             }
                //                         }
                //                     }
                //                     button {
                //                         class: "shrink-0 rounded border border-border/40 px-2 py-1 text-xs hover:bg-foreground/5",
                //                         onclick: move |_| {
                //                             goal_edit.set(Some(g.clone()));
                //                             show_goal_form.set(true);
                //                         },
                //                         "Изменить"
                //                     }
                //                 }
                //                 div { class: "mt-2 text-[10px] text-foreground/45",
                //                     if g.completed { "Статус: выполнено" } else { "Статус: в работе" }
                //                     " · порог "
                //                     "{g.ordering.as_str()}"
                //                 }
                //             }
                //         }
                //     }
                // }
                div {
                    div { class: "flex max-h-64 flex-col gap-2 overflow-auto pr-1",
                        if jobs().is_empty() {
                            div { class: "text-sm text-foreground/55",
                                "Задач нет — добавьте из календаря или здесь."
                            }
                        }

                        for (j, edit_job, delete_id) in jobs_list.iter().cloned() {
                            div { class: "rounded-md border border-border/30 p-3",
                                div { class: "flex items-center justify-between gap-2",
                                    div {
                                        div { class: "text-sm font-medium text-foreground",
                                            "{j.name}"
                                        }
                                        div { class: "mt-2 flex flex-wrap gap-1",
                                            for t in j.tags.iter().cloned() {
                                                span { class: "rounded-full border border-border/40 px-2 py-0.5 text-[10px] text-foreground/80",
                                                    "{t.name}"
                                                }
                                            }
                                        }
                                    }
                                    div { class: "flex gap-1",
                                        button {
                                            class: "rounded border border-border/40 px-2 py-1 text-xs hover:bg-foreground/5",
                                            onclick: move |_| {
                                                job_edit.set(Some(edit_job.clone()));
                                                show_job_form.set(true);
                                            },
                                            "Изменить"
                                        }

                                        button {
                                            class: "rounded border border-border/40 px-2 py-1 text-xs hover:bg-foreground/5",
                                            onclick: move |_| {
                                                delete(delete_id);
                                                info("Задача успешно удалена".to_string());
                                            },
                                            Icon { icon: LdTrash }
                                        }
                                    }
                                
                                }
                            }
                        }
                    }
                }
            }
        }

        if show_goal_form() {
            div {
                class: "fixed inset-0 z-[250] flex items-center justify-center bg-black/50 p-4",
                onclick: move |_| show_goal_form.set(false),
                div {
                    class: "max-h-[90vh] w-full max-w-lg overflow-y-auto rounded-lg border border-border/40 bg-background p-6 shadow-xl",
                    onclick: move |e| e.stop_propagation(),
                    button {
                        class: "mb-2 float-right rounded px-2 text-sm text-foreground/60 hover:bg-foreground/10",
                        onclick: move |_| show_goal_form.set(false),
                        "✕"
                    }
                    GoalForm {
                        day,
                        goal: goal_edit(),
                        on_save: Callback::new(move |g: GoalModel| {
                            spawn(async move {
                                let _ = tokio::task::spawn_blocking(move || {
                                        with_database_mut(|db| {
                                            if g.id.is_some() {
                                                db.update_goal(&g)
                                            } else {
                                                db.insert_goal(&g).map(|_| ())
                                            }
                                        })
                                    })
                                    .await;
                            });
                            show_goal_form.set(false);
                        }),
                        on_cancel: Callback::new(move |_| show_goal_form.set(false)),
                    }
                }
            }
        }

        if show_job_form() {
            div {
                class: "fixed inset-0 z-[250] flex items-center justify-center bg-black/50 p-4",
                onclick: move |_| show_job_form.set(false),
                div {
                    class: "max-h-[90vh] w-full max-w-lg overflow-y-auto rounded-lg border border-border/40 bg-background p-6 shadow-xl",
                    onclick: move |e| e.stop_propagation(),
                    button {
                        class: "mb-2 float-right rounded px-2 text-sm text-foreground/60 hover:bg-foreground/10",
                        onclick: move |_| show_job_form.set(false),
                        "✕"
                    }
                    JobForm {
                        day: job_edit().as_ref().map(job_anchor_day).unwrap_or(day),
                        job: job_edit(),
                        start_ts: 9 * 3600,
                        end_ts: 18 * 3600,
                        on_save: Callback::new(move |job: JobModel| {
                            update(job.clone());
                            show_job_form.set(false);
                        }),
                        on_cancel: Callback::new(move |_| show_job_form.set(false)),
                    }
                }
            }
        }
    }
}

#[component]
fn MetricCard(title: String, value: String, hint: String) -> Element {
    rsx! {
        div { class: "rounded-md border border-border/40 bg-background/70 p-4",
            div { class: "text-xs text-foreground/55", "{title}" }
            div { class: "mt-1 truncate text-lg font-semibold text-foreground", "{value}" }
            div { class: "mt-1 text-xs text-foreground/50", "{hint}" }
        }
    }
}

#[component]
fn InfoRow(label: String, value: String) -> Element {
    rsx! {
        div { class: "flex items-center justify-between gap-3 rounded-md bg-foreground/5 px-3 py-2",
            span { class: "text-sm text-foreground/60", "{label}" }
            span { class: "text-sm font-medium text-foreground", "{value}" }
        }
    }
}

#[component]
fn DailyUsageChart(days: Vec<DayUsage>) -> Element {
    let max_ms = days.iter().map(|day| day.total_ms).max().unwrap_or(0);
    let visible_days = days.iter().rev().take(45).cloned().collect::<Vec<_>>();

    if visible_days.is_empty() || max_ms == 0 {
        return rsx! {
            div { class: "flex h-72 items-center justify-center rounded-md border border-dashed border-border/40 bg-foreground/[0.03] text-sm text-foreground/55",
                "Нет данных для графика"
            }
        };
    }

    rsx! {
        div { class: "relative h-80 overflow-hidden rounded-md border border-border/30 bg-foreground/[0.03] p-4",
            div { class: "pointer-events-none absolute inset-x-4 top-4 bottom-10 grid grid-rows-4",
                for _ in 0..4 {
                    div { class: "border-t border-border/25" }
                }
            }

            div { class: "relative flex h-full items-end gap-2 pb-8 pt-6 overflow-visible",
                for day in visible_days.into_iter().rev() {
                    DailyUsageBar { day, max_ms }
                }
            }

            div { class: "pointer-events-none absolute bottom-3 left-4 right-4 flex justify-between text-[10px] text-foreground/35",
                span { "дни" }
                span { "последние 45" }
            }
        }
    }
}

#[component]
fn DailyUsageBar(day: DayUsage, max_ms: u64) -> Element {
    let bar_height = if max_ms > 0 {
        ((day.total_ms as f64 / max_ms as f64) * 220.0).max(8.0)
    } else {
        0.0
    };
    let active_height = if day.total_ms > 0 {
        (day.active_ms as f64 / day.total_ms as f64) * bar_height
    } else {
        0.0
    };
    let idle_height = (bar_height - active_height).max(0.0);

rsx! {
        div { class: "group relative flex h-[250px] min-w-8 flex-1 flex-col items-center justify-start gap-2 ",

            Tooltip {
                class: "flex items-end justify-center max-w-[36px]",
                align: TooltipAlign::Right,
                gap: 0,
                target: rsx! {
                    div { class: "",
                        div { class: "font-semibold text-foreground", "{day.date.format(\"%d.%m.%Y\")}" }
                        div { class: "mt-1 text-foreground/65", "Всего: {format_duration_short(day.total_ms)}" }
                        div { class: "text-foreground/65", "Активно: {format_duration_short(day.active_ms)}" }
                        div { class: "text-foreground/65", "Idle: {format_duration_short(day.idle_ms)}" }
                        div { class: "truncate text-foreground/65", "Топ: {day.top_app}" }
                    }
                },
                div {
                    class: "flex w-full max-w-9 flex-col justify-start items-center  rounded-t-md rounded-b-sm border border-primary/20 bg-foreground/10 shadow-sm transition-transform",
                    style: "height: {bar_height}px;",
                    div {
                        class: "w-full bg-primary/25",
                        style: "height: {idle_height}px;",
                    }
                    div {
                        class: "w-full bg-primary",
                        style: "height: {active_height}px;",
                    }
                }
            
            }

            div { class: "h-4 text-[10px] text-foreground/45", "{day.date.format(\"%d.%m\")}" }
        }
    }
}

#[component]
fn HourActivityChart(hours: Vec<HourUsage>) -> Element {
    let max_ms = hours.iter().map(|hour| hour.total_ms).max().unwrap_or(0);

    if max_ms == 0 {
        return rsx! {
            div { class: "flex h-48 items-center justify-center rounded-md border border-dashed border-border/40 bg-foreground/[0.03] text-sm text-foreground/55",
                "Нет данных по часам"
            }
        };
    }

    rsx! {
        div { class: "grid gap-2 sm:grid-cols-6 xl:grid-cols-12",
            for hour in hours {
                HourActivityCell { hour, max_ms }
            }
        }
    }
}

#[component]
fn HourActivityCell(hour: HourUsage, max_ms: u64) -> Element {
    let (intensity, fill_height, active_percent, alpha) = if max_ms > 0 {
        let intensity = (hour.total_ms as f64 / max_ms as f64).clamp(0.0, 1.0);
        let fill_height = (intensity * 72.0).max(if hour.total_ms > 0 { 6.0 } else { 0.0 });
        let active_percent = if hour.total_ms > 0 {
            (hour.active_ms as f64 / hour.total_ms as f64) * 100.0
        } else {
            0.0
        };
        let alpha = 0.08 + intensity * 0.28;
        (intensity, fill_height, active_percent, alpha)
    } else {
        (0.0, 0.0, 0.0, 0.08)
    };

    rsx! {

        div { class: "group rounded-md border border-border/30 bg-foreground/[0.03] p-2 transition-colors hover:border-primary/50 hover:bg-primary/5",
            div { class: "mb-2 flex items-center justify-between gap-1",
                span { class: "text-xs font-medium text-foreground/65", "{hour.hour:02}" }
                span { class: "text-[10px] text-foreground/40", "{active_percent:.0}%" }
            }
            div { class: "flex h-20 items-end overflow-hidden rounded-sm bg-foreground/10",
                div {
                    class: "w-full rounded-t-sm bg-primary transition-all",
                    style: "height: {fill_height}px; opacity: {alpha};",
                }
            }
            div { class: "mt-2 truncate text-xs font-medium text-foreground",
                "{format_duration_short(hour.total_ms)}"
            }
        }
    }
}

#[component]
fn DailyBar(day: DayUsage, max_ms: u64) -> Element {
    let (height, active_height) = if max_ms > 0 {
        let h = ((day.total_ms as f64 / max_ms as f64) * 100.0).max(2.0);
        let ah = if day.total_ms > 0 {
            (day.active_ms as f64 / day.total_ms as f64) * 100.0
        } else {
            0.0
        };
        (h, ah)
    } else {
        (0.0, 0.0)
    };

     rsx! {
        div { class: "group flex min-w-8 flex-1 flex-col items-center justify-end gap-1",
            Tooltip {
                align: TooltipAlign::Right,
                target: rsx! {
                    div { class: "text-xs",
                        div { class: "font-medium text-foreground", "{day.date.format(\"%d.%m.%Y\")}" }
                        div { class: "text-foreground/65", "Всего: {format_duration_short(day.total_ms)}" }
                        div { class: "text-foreground/65", "Топ: {day.top_app}" }
                    }

                },
                div {
                    class: "relative flex w-full max-w-10 items-end overflow-hidden rounded-sm bg-foreground/10",
                    style: "height: {height}%;",
                    div {
                        class: "w-full bg-primary/80",
                        style: "height: {active_height}%;",
                    }
                }
            }
            span { class: "text-[10px] text-foreground/45", "{day.date.format(\"%d.%m\")}" }
        }
    }
}

#[component]
fn HourCell(hour: HourUsage, max_ms: u64) -> Element {
    let intensity = if max_ms > 0 {
        (hour.total_ms as f64 / max_ms as f64).clamp(0.0, 1.0)
    } else {
        0.0
    };
    let opacity = 0.12 + intensity * 0.78;

    rsx! {
        div { class: "rounded-md border border-border/30 p-2",
            div { class: "mb-2 text-xs text-foreground/55", "{hour.hour:02}:00" }
            div {
                class: "h-12 rounded-sm bg-primary",
                style: "opacity: {opacity};",
            }
            div { class: "mt-2 text-xs font-medium text-foreground",
                "{format_duration_short(hour.total_ms)}"
            }
        }
    }
}

#[component]
fn AppRow(app: AppUsage, max_ms: u64) -> Element {
   let width = if max_ms > 0 {
        ((app.total_ms() as f64 / max_ms as f64) * 100.0).max(2.0)
    } else {
        0.0
    };

    rsx! {
        div { class: "rounded-md border border-border/30 p-3",
            div { class: "mb-2 flex items-center justify-between gap-3",
                div { class: "flex min-w-0 items-center gap-2",
                    if let Some(icon) = app.icon.clone() {
                        img { class: "h-5 w-5 rounded-sm", src: icon }
                    }
                    span { class: "truncate text-sm font-medium text-foreground", "{app.name}" }
                }
                span { class: "text-sm text-foreground/70", "{format_duration_short(app.total_ms())}" }
            }
            div { class: "h-2 overflow-hidden rounded-sm bg-foreground/10",
                div { class: "h-full bg-primary", style: "width: {width}%;" }
            }
            div { class: "mt-2 flex flex-wrap gap-x-4 gap-y-1 text-xs text-foreground/55",
                span { "active {format_duration_short(app.active_ms)}" }
                span { "idle {format_duration_short(app.idle_ms)}" }
                span { "{app.events} событий" }
                span { "{app.days} дней" }
            }
        }
    }
}

#[component]
fn DayRow(day: DayUsage) -> Element {
    rsx! {
        div { class: "rounded-md border border-border/30 p-3",
            div { class: "flex items-center justify-between gap-3",
                div {
                    div { class: "text-sm font-medium text-foreground",
                        "{day.date.format(\"%d.%m.%Y\")}"
                    }
                    div { class: "text-xs text-foreground/55",
                        "Топ приложение: {day.top_app}"
                    }
                }
                div { class: "text-right",
                    div { class: "text-sm font-semibold text-foreground",
                        "{format_duration_short(day.total_ms)}"
                    }
                    div { class: "text-xs text-foreground/55", "{day.events} событий" }
                }
            }
            div { class: "mt-2 grid grid-cols-2 gap-2 text-xs text-foreground/55",
                div { "Активно: {format_duration_short(day.active_ms)}" }
                div { class: "text-right",
                    "Бездействие: {format_duration_short(day.idle_ms)}"
                }
            }
        }
    }
}

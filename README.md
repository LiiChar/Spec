# Spec

> Desktop activity tracking for Windows, built with Rust and Dioxus.

**Spec** is a native desktop application that observes active window usage on Windows, converts raw foreground-window changes into structured activity intervals, stores them in SQLite, and renders them in a real-time UI.

Instead of recording noisy polling ticks, Spec models **continuous periods of activity** — making the collected data useful for inspection, analysis, and future productivity tooling.


## Why Spec?

Most activity trackers either collect raw event streams or behave like opaque black boxes.

Spec is built around a simple idea:

> **turn operating-system observations into meaningful time intervals**

That makes it useful for:

* understanding how time is actually spent
* building timeline-based visualizations
* aggregating application usage
* experimenting with analytics and behavioral tooling
* serving as a foundation for future desktop productivity systems


### Features

* active foreground window tracking
* automatic window switch detection
* idle state detection
* event persistence with SQLite
* real-time UI updates
* daily timeline visualization
* activity list view
* lightweight architecture with clear data boundaries


## Architecture

The project is centered around a single pipeline:

```text
Windows API → tracker → channels → database + UI
```

### Runtime flow

1. A background tracker periodically polls the current foreground window.
2. The tracker compares current activity with previous activity.
3. When activity changes, the previous interval is finalized.
4. An `EventModel` is emitted.
5. The event is duplicated through channels:

   * persisted to SQLite
   * pushed to UI
6. The UI loads historical events from the database and listens for live updates.



## Screenshots

> Screenshots will be added soon.

Recommended future additions:

* timeline view
* event list
* real-time tracking demo


## Technology stack

* **Rust**
* **Dioxus 0.7.1**
* **SQLite**
* **Windows API**


## Getting started

### Requirements

* **Windows**
* **Rust stable**
* **cargo**
* **dx cli**


### Clone

```bash
git clone https://github.com/LiiChar/Spec
cd spec
```

### Build

```bash
dx build
```

### Run

```bash
dx server
```

## Roadmap

### Near-term

* richer filtering
* process grouping
* better statistics
* improved idle heuristics
* stronger migrations
* polished UI interactions

### Mid-term

* weekly and monthly analytics
* application usage summaries
* tagging and categorization
* export capabilities
* session-level analysis

### Long-term

* productivity insights
* behavioral pattern detection
* rule-based automation
* cross-platform abstraction

Доработки

1. Цвет на таймолайне из цвета иконки, изменение цвета - V
2. Добавление тегов к приложениям, добавление автоматического тегирования по условиям regex
3. Добавить проверку на границу экрна, для предотвращения скпрытия части элемента за экраном - V
4. Режишть изменить unknown или вообще его не учитывать
5. Доработать интернализацию
6. Исправить выбор дат в статистике
7. Доработать выбор времени в задачах
8. Подумать над настройками
9. Исправить загрузку шаблонов тегов
10. Уменьшить вес и нагрузку приложения - Вес уменьшен до 6 Мб при dx build --release, осталось уменьшить нагрузку
11. Исправить стил glass
12. Добавить минуты при выборе часа для более точного отображения времени - V
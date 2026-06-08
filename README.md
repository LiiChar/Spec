# Spexe

> Desktop activity tracking for Windows, built with Rust and Dioxus.

**Spexe** is a native desktop application that observes active window usage on Windows, converts raw foreground-window changes into structured activity intervals, stores them in SQLite, and renders them in a real-time UI.

Instead of recording noisy polling ticks, Spexe models **continuous periods of activity** — making the collected data useful for inspection, analysis, and future productivity tooling.


## Why Spexe?

Most activity trackers either collect raw event streams or behave like opaque black boxes.

Spexe is built around a simple idea:

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
git clone https://github.com/LiiChar/Spexe
cd spexe
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

Я бы немного структурировал и уточнил список, чтобы потом было проще вести roadmap и не потерять детали.


### ❌ 3. Доработать Glass-стиль

### ❌ 4. Продумать настройки приложения

Возможно разделить на категории:

* Общие
* Внешний вид
* Таймлайн
* Задачи
* Уведомления
* Теги
* Производительность
* Резервное копирование

Поля:

1. Мягкие цвета для событий
2. Отображать теги
3. Тип отображения тегов, круги или палки
4. Высота невыбранного сегмена - по умлочанию 800
5. Высота выбранного сегмента - по умлочанию 800

### ❌ 5. Автоматическое тегирование приложений

### ❌ 6. Форма создания тегов

### ❌ 8. Исправить загрузку шаблонов тегов

### ✅ 9. Исправить выбор дат в статистике


### ❌ 10. Доработать выбор времени в задачах

### ❌ 11. Добавить минуты при выборе времени

* Выбор с шагом 1/5/15 минут.
* Настраиваемый шаг времени.

### ❌ 15. Доработать интернационализацию

## Логика данных

### ❌ 16. Пересмотреть статус `Unknown`

Варианты:

* Полностью убрать.
* Использовать как технический статус.
* Автоматически заменять на "Без категории".
* Скрывать из интерфейса и статистики.

17. Добавить в timeline выбор промежутков
18. Получение общей статистики работы компльютера, время запуска компьютера, продолжительность сессии 
19. Отображение тегов рядома с названием события в timeline в виде цветной палочки, при наведении отображается название
20. Добавить автоматический просчёт высоты текста в событиях на таймлайне
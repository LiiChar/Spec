use chrono::{Datelike, NaiveDateTime, Timelike};

#[derive(Debug, Clone)]
pub enum Field {
    Any,
    Value(u32),
    Range(u32, u32),
    Step(u32),
    List(Vec<Field>),
}

#[derive(Debug, Clone)]
pub struct CronExpr {
    pub minute: Field,
    pub hour: Field,
    pub day: Field,
    pub month: Field,
    pub weekday: Field,
}

fn parse_field(input: &str) -> Field {
    if input == "*" {
        return Field::Any;
    }

    if input.contains(',') {
        return Field::List(input.split(',').map(parse_field).collect());
    }

    if let Some(rest) = input.strip_prefix("*/") {
        return Field::Step(rest.parse().unwrap());
    }

    if let Some((a, b)) = input.split_once('-') {
        return Field::Range(a.parse().unwrap(), b.parse().unwrap());
    }

    Field::Value(input.parse().unwrap())
}

impl CronExpr {
    pub fn parse(expr: &str) -> Self {
        let parts: Vec<&str> = expr.split_whitespace().collect();
        assert_eq!(parts.len(), 5, "Invalid cron format");

        Self {
            minute: parse_field(parts[0]),
            hour: parse_field(parts[1]),
            day: parse_field(parts[2]),
            month: parse_field(parts[3]),
            weekday: parse_field(parts[4]),
        }
    }
}

impl Field {
    pub fn matches(&self, v: u32) -> bool {
        match self {
            Field::Any => true,
            Field::Value(x) => *x == v,
            Field::Range(a, b) => v >= *a && v <= *b,
            Field::Step(step) => v % step == 0,
            Field::List(list) => list.iter().any(|f| f.matches(v)),
        }
    }
}

impl CronExpr {
    pub fn matches(&self, dt: NaiveDateTime) -> bool {
        let weekday = dt.weekday().num_days_from_sunday();

        self.minute.matches(dt.minute())
            && self.hour.matches(dt.hour())
            && self.day.matches(dt.day())
            && self.month.matches(dt.month())
            && self.weekday.matches(weekday)
    }

    pub fn is_today_match(&self, dt: NaiveDateTime) -> bool {
        let weekday = dt.weekday().num_days_from_sunday();

        self.day.matches(dt.day())
            && self.month.matches(dt.month())
            && self.weekday.matches(weekday)
    }
}

impl CronExpr {
    pub fn next_match(&self, mut dt: NaiveDateTime) -> Option<NaiveDateTime> {
        for _ in 0..(60 * 24 * 365) {
            if self.matches(dt) {
                return Some(dt);
            }

            dt = dt + chrono::Duration::minutes(1);
        }

        None
    }
}

pub struct CronBuilder {
    minute: String,
    hour: String,
    day: String,
    month: String,
    weekday: String,
}

impl CronBuilder {
    pub fn new() -> Self {
        Self {
            minute: "*".into(),
            hour: "*".into(),
            day: "*".into(),
            month: "*".into(),
            weekday: "*".into(),
        }
    }

    // === Простые расписания ===
    pub fn every_day(mut self) -> Self {
        self.hour = "0".into();
        self.minute = "0".into();
        self
    }

    pub fn every_hour(mut self) -> Self {
        self.minute = "0".into();
        self
    }

    pub fn every_month(mut self) -> Self {
        self.day = "1".into();
        self.hour = "0".into();
        self.minute = "0".into();
        self
    }

    pub fn every_weekday(mut self) -> Self {
        // Пн-Пт
        self.weekday = "1-5".into();
        self.hour = "0".into();
        self.minute = "0".into();
        self
    }

    // === Каждые N ===
    pub fn every_n_minutes(mut self, n: u32) -> Self {
        self.minute = format!("*/{}", n);
        self
    }

    pub fn every_n_hours(mut self, n: u32) -> Self {
        self.hour = format!("*/{}", n);
        self.minute = "0".into();
        self
    }

    pub fn every_n_days(mut self, n: u32) -> Self {
        self.day = format!("*/{}", n);
        self.hour = "0".into();
        self.minute = "0".into();
        self
    }

    pub fn every_n_months(mut self, n: u32) -> Self {
        self.month = format!("*/{}", n);
        self.day = "1".into();
        self.hour = "0".into();
        self.minute = "0".into();
        self
    }

    // === Списки и диапазоны ===
    pub fn minutes(mut self, minutes: &[u32]) -> Self {
        self.minute = Self::list_or_range(minutes);
        self
    }

    pub fn hours(mut self, hours: &[u32]) -> Self {
        self.hour = Self::list_or_range(hours);
        self
    }

    pub fn days(mut self, days: &[u32]) -> Self {
        self.day = Self::list_or_range(days);
        self
    }

    pub fn months(mut self, months: &[u32]) -> Self {
        self.month = Self::list_or_range(months);
        self
    }

    pub fn weekdays(mut self, weekdays: &[u32]) -> Self {
        self.weekday = Self::list_or_range(weekdays);
        self
    }

    // === Вспомогательный метод ===
    fn list_or_range(values: &[u32]) -> String {
        if values.is_empty() {
            return "*".into();
        }
        if values.len() == 1 {
            return values[0].to_string();
        }

        // Проверяем, является ли это последовательным диапазоном
        let min = *values.iter().min().unwrap();
        let max = *values.iter().max().unwrap();

        if values.len() == (max - min + 1) as usize
            && values.iter().all(|&v| (min..=max).contains(&v))
        {
            format!("{}-{}", min, max)
        } else {
            values
                .iter()
                .map(|&v| v.to_string())
                .collect::<Vec<_>>()
                .join(",")
        }
    }

    pub fn at(mut self, hour: u32, minute: u32) -> Self {
        self.hour = hour.to_string();
        self.minute = minute.to_string();
        self
    }

    pub fn build(self) -> CronExpr {
        CronExpr::parse(&format!(
            "{} {} {} {} {}",
            self.minute, self.hour, self.day, self.month, self.weekday
        ))
    }
}

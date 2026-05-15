use chrono::{
    DateTime, Datelike, Duration, Months, TimeZone, Timelike,
};
use dioxus::html::u::is;
use std::{
    future::Future,
    pin::Pin,
    str::FromStr,
    sync::Arc,
};



/// Ошибки cron parser.
///
/// Возможные варианты:
///
/// - InvalidFormat:
///   Неверное количество полей.
///
/// - InvalidValue:
///   Значение вышло за допустимый диапазон.
///
/// Пример:
/// ```rust
/// let cron = CronExpr::parse("* *");
/// ```
#[derive(Debug, Clone)]
pub enum CronError {
    InvalidFormat(String),
    InvalidValue(String),
}

/// Описание одного cron поля.
///
/// Поддерживаемые типы:
///
/// Any:
/// ```cron
/// *
/// ```
///
/// Value:
/// ```cron
/// 5
/// ```
///
/// Range:
/// ```cron
/// 1-10
/// ```
///
/// Step:
/// ```cron
/// */5
/// ```
///
/// RangeStep:
/// ```cron
/// 1-10/2
/// ```
///
/// List:
/// ```cron
/// 1,2,3
/// ```
///
/// Quartz extensions:
///
/// LastDay:
/// ```cron
/// L
/// ```
///
/// NearestWeekday:
/// ```cron
/// 15W
/// ```
///
/// NthWeekday:
/// ```cron
/// MON#2
/// ```
#[derive(Debug, Clone)]
pub enum Field {
    Any,
    Value(u32),
    Range(u32, u32),
    Step(u32),
    RangeStep(u32, u32, u32),
    List(Vec<Field>),

    /// Последний день месяца.
    ///
    /// Пример:
    /// ```cron
    /// L
    /// ```
    LastDay,

    /// Ближайший рабочий день.
    ///
    /// Пример:
    /// ```cron
    /// 15W
    /// ```
    NearestWeekday(u32),

    /// N-й weekday месяца.
    ///
    /// Пример:
    /// ```cron
    /// MON#2
    /// ```
    NthWeekday(u32, u32),
}

impl Field {
    /// Проверяет соответствует ли значение полю.
    ///
    /// Примеры:
    ///
    /// ```rust
    /// Field::Any.matches(10);
    /// ```
    ///
    /// ```rust
    /// Field::Value(5).matches(5);
    /// ```
    ///
    /// ```rust
    /// Field::Range(1, 10).matches(7);
    /// ```
    ///
    /// ```rust
    /// Field::Step(5).matches(10);
    /// ```
    pub fn matches(&self, value: u32) -> bool {
        match self {
            Field::Any => true,

            Field::Value(v) => *v == value,

            Field::Range(a, b) => {
                value >= *a && value <= *b
            }

            Field::Step(step) => value % *step == 0,

            Field::RangeStep(start, end, step) => {
                value >= *start
                    && value <= *end
                    && (value - *start) % *step == 0
            }

            Field::List(list) => {
                list.iter().any(|f| f.matches(value))
            }

            // Специальные Quartz поля
            // проверяются отдельно.
            Field::LastDay => false,
            Field::NearestWeekday(_) => false,
            Field::NthWeekday(_, _) => false,
        }
    }
}


/// Основная cron expression структура.
///
/// Формат:
///
/// ```text
/// second minute hour day month weekday year
/// ```
///
/// Пример:
///
/// ```cron
/// 0 */5 * * * * *
/// ```
///
/// Alias:
///
/// ```cron
/// @daily
/// @weekly
/// @hourly
/// ```
///
/// Month aliases:
///
/// ```cron
/// JAN FEB MAR
/// ```
///
/// Weekday aliases:
///
/// ```cron
/// MON TUE WED
/// ```
#[derive(Debug, Clone)]
pub struct CronExpr {
    pub second: Field,
    pub minute: Field,
    pub hour: Field,
    pub day: Field,
    pub month: Field,
    pub weekday: Field,
    pub year: Field,

    /// Timezone offset.
    ///
    /// По умолчанию UTC.
    pub timezone: chrono::FixedOffset,
}

impl CronExpr {
    /// Парсит cron expression.
    ///
    /// Поддерживает:
    ///
    /// - Quartz format
    /// - aliases
    /// - ranges
    /// - lists
    /// - steps
    /// - month aliases
    /// - weekday aliases
    ///
    /// Примеры:
    ///
    /// ```rust
    /// let cron =
    ///     CronExpr::parse(
    ///         "0 */5 * * * * *"
    ///     ).unwrap();
    /// ```
    ///
    /// ```rust
    /// let cron =
    ///     CronExpr::parse("@daily")
    ///         .unwrap();
    /// ```
    ///
    /// ```rust
    /// let cron =
    ///     CronExpr::parse(
    ///         "0 0 12 L * * *"
    ///     ).unwrap();
    /// ```
    pub fn parse(
        input: &str,
    ) -> Result<Self, CronError> {
        let input = Self::expand_alias(input);

        let parts: Vec<&str> =
            input.split_whitespace().collect();

        if parts.len() != 7 {
            return Err(
                CronError::InvalidFormat(
                    "Expected 7 fields".into(),
                ),
            );
        }

        Ok(Self {
            second: parse_field(parts[0])?,
            minute: parse_field(parts[1])?,
            hour: parse_field(parts[2])?,
            day: parse_field(parts[3])?,
            month: parse_field(parts[4])?,
            weekday: parse_field(parts[5])?,
            year: parse_field(parts[6])?,
            timezone:
                chrono::FixedOffset::east_opt(0)
                    .unwrap(),
        })
    }

    /// Устанавливает timezone offset.
    ///
    /// Пример:
    ///
    /// ```rust
    /// let cron = CronExpr::parse("@daily")
    ///     .unwrap()
    ///     .with_timezone(3);
    /// ```
    ///
    /// GMT+3:
    /// ```rust
    /// .with_timezone(3)
    /// ```
    ///
    /// GMT-5:
    /// ```rust
    /// .with_timezone(-5)
    /// ```
    pub fn with_timezone(
        mut self,
        offset_hours: i32,
    ) -> Self {
        self.timezone =
            chrono::FixedOffset::east_opt(
                offset_hours * 3600,
            )
            .unwrap();

        self
    }

    /// Раскрывает cron aliases.
    ///
    /// Например:
    ///
    /// ```cron
    /// @daily
    /// ```
    ///
    /// превращается в:
    ///
    /// ```cron
    /// 0 0 0 * * * *
    /// ```
    fn expand_alias(input: &str) -> String {
        match input {
            "@hourly" => "0 0 * * * * *".into(),

            "@daily" => {
                "0 0 0 * * * *".into()
            }

            "@weekly" => {
                "0 0 0 * * 0 *".into()
            }

            "@monthly" => {
                "0 0 0 1 * * *".into()
            }

            "@yearly" => {
                "0 0 0 1 1 * *".into()
            }

            _ => input.into(),
        }
    }

    /// Проверяет совпадает ли день месяца.
    ///
    /// Используется для:
    ///
    /// - L
    /// - W
    /// - обычных day expressions
    fn matches_day<Tz: TimeZone>(
        &self,
        dt: DateTime<Tz>,
    ) -> bool {
        match &self.day {
            Field::LastDay => {
                let next_day = dt
                    .clone()
                    .checked_add_signed(
                        Duration::days(1),
                    );

                match next_day {
                    Some(next) => {
                        next.month() != dt.month()
                    }

                    None => false,
                }
            }

            Field::NearestWeekday(day) => {
                nearest_weekday(
                    dt.year(),
                    dt.month(),
                    *day,
                ) == dt.day()
            }

            other => other.matches(dt.day()),
        }
    }

    /// Проверяет совпадает ли weekday.
    ///
    /// Используется для:
    ///
    /// - MON#2
    /// - обычных weekday expressions
    fn matches_weekday<Tz: TimeZone>(
        &self,
        dt: DateTime<Tz>,
    ) -> bool {
        match &self.weekday {
            Field::NthWeekday(weekday, nth) => {
                is_nth_weekday_of_month(
                    dt,
                    *weekday,
                    *nth,
                )
            }

            other => {
                other.matches(
                    dt.weekday()
                        .num_days_from_sunday(),
                )
            }
        }
    }

    /// Проверяет соответствует ли дата cron expression.
    ///
    /// Это основная функция проверки.
    ///
    /// Пример:
    ///
    /// ```rust
    /// use chrono::Local;
    ///
    /// let cron =
    ///     CronExpr::parse(
    ///         "0 */5 * * * * *"
    ///     ).unwrap();
    ///
    /// let matched =
    ///     cron.matches(Local::now());
    /// ```
    ///
    /// Проверка текущего времени:
    ///
    /// ```rust
    /// if cron.matches(Local::now()) {
    ///     println!("RUN");
    /// }
    /// ```
    pub fn matches<Tz: TimeZone>(
        &self,
        dt: DateTime<Tz>,
        filter: Option<String>,
    ) -> bool {
        let binding = filter.unwrap_or(String::from("+ + + + + + +"));
        let filter = binding.split(" ").collect::<Vec<&str>>();

        let mut is_match = true;

        if filter.get(0).unwrap_or(&"+").to_string() == "+" {
            is_match = self.second.matches(dt.second());
        };

        if filter.get(1).unwrap_or(&"+").to_string() == "+" {
            is_match = is_match && self.minute.matches(dt.minute());
        };

        if filter.get(2).unwrap_or(&"+").to_string() == "+" {
            is_match = is_match && self.hour.matches(dt.hour());
        };

        if filter.get(3).unwrap_or(&"+").to_string() == "+" {
            is_match = is_match && self.matches_day(dt.clone());
        };

        if filter.get(4).unwrap_or(&"+").to_string() == "+" {
            is_match = is_match && self.month.matches(dt.month());
        };

        if filter.get(5).unwrap_or(&"+").to_string() == "+" {
            is_match = is_match && self.matches_weekday(dt.clone());
        };

        if filter.get(6).unwrap_or(&"+").to_string() == "+" {
            is_match = is_match && self.year.matches(dt.year() as u32);
        };

        is_match
    }

    /// Ищет следующую дату подходящую
    /// под cron expression.
    ///
    /// Оптимизирован:
    ///
    /// - прыжки по годам
    /// - месяцам
    /// - дням
    /// - часам
    /// - минутам
    ///
    /// без полного перебора секунд.
    ///
    /// Пример:
    ///
    /// ```rust
    /// use chrono::Local;
    ///
    /// let cron =
    ///     CronExpr::parse(
    ///         "0 */5 * * * * *"
    ///     ).unwrap();
    ///
    /// let next =
    ///     cron.next_match(
    ///         Local::now()
    ///     );
    /// ```
    pub fn next_match<Tz: TimeZone>(
        &self,
        from: DateTime<Tz>,
    ) -> Option<DateTime<Tz>> {
        let mut dt =
            from + Duration::seconds(1);

        for _ in 0..100000 {
            if !self.year.matches(
                dt.year() as u32,
            ) {
                dt = dt
                    .with_year(dt.year() + 1)?
                    .with_month(1)?
                    .with_day(1)?
                    .with_hour(0)?
                    .with_minute(0)?
                    .with_second(0)?;

                continue;
            }

            if !self.month.matches(
                dt.month(),
            ) {
                dt = dt
                    .checked_add_months(
                        Months::new(1),
                    )?
                    .with_day(1)?
                    .with_hour(0)?
                    .with_minute(0)?
                    .with_second(0)?;

                continue;
            }

            if !self.day.matches(dt.day()) {
                dt = dt
                    .checked_add_signed(
                        Duration::days(1),
                    )?
                    .with_hour(0)?
                    .with_minute(0)?
                    .with_second(0)?;

                continue;
            }

            if !self.hour.matches(
                dt.hour(),
            ) {
                dt = dt
                    .checked_add_signed(
                        Duration::hours(1),
                    )?
                    .with_minute(0)?
                    .with_second(0)?;

                continue;
            }

            if !self.minute.matches(
                dt.minute(),
            ) {
                dt = dt
                    .checked_add_signed(
                        Duration::minutes(1),
                    )?
                    .with_second(0)?;

                continue;
            }

            if !self.second.matches(
                dt.second(),
            ) {
                dt = dt
                    .checked_add_signed(
                        Duration::seconds(1),
                    )?;

                continue;
            }

            return Some(dt);
        }

        None
    }

    /// Валидирует cron expression.
    ///
    /// Проверяет диапазоны:
    ///
    /// seconds: 0-59
    /// minutes: 0-59
    /// hours: 0-23
    /// days: 1-31
    /// months: 1-12
    /// weekdays: 0-6
    ///
    /// Пример:
    ///
    /// ```rust
    /// cron.validate()?;
    /// ```
    pub fn validate(
        &self,
    ) -> Result<(), CronError> {
        validate_field(
            &self.second,
            0,
            59,
            "seconds",
        )?;

        validate_field(
            &self.minute,
            0,
            59,
            "minutes",
        )?;

        validate_field(
            &self.hour,
            0,
            23,
            "hours",
        )?;

        validate_field(
            &self.day,
            1,
            31,
            "days",
        )?;

        validate_field(
            &self.month,
            1,
            12,
            "months",
        )?;

        validate_field(
            &self.weekday,
            0,
            6,
            "weekday",
        )?;

        Ok(())
    }

    /// Конвертирует cron expression
    /// в человекочитаемый вид.
    ///
    /// Пример:
    ///
    /// ```text
    /// Каждую 5 минуту
    /// ```
    ///
    /// Сейчас реализация простая,
    /// но её можно сильно улучшить.
    pub fn to_human(&self) -> String {
        format!(
            "Каждую {:?} секунду, {:?} минуту, {:?} час",
            self.second,
            self.minute,
            self.hour
        )
    }
}

fn validate_field(
    field: &Field,
    min: u32,
    max: u32,
    name: &str,
) -> Result<(), CronError> {
    match field {
        Field::Value(v) => {
            if *v < min || *v > max {
                return Err(CronError::InvalidValue(format!(
                    "{} out of range",
                    name
                )));
            }
        }

        Field::Range(a, b) => {
            if *a < min || *b > max {
                return Err(CronError::InvalidValue(format!(
                    "{} out of range",
                    name
                )));
            }
        }

        _ => {}
    }

    Ok(())
}

fn parse_month(value: &str) -> Option<u32> {
    match value.to_uppercase().as_str() {
        "JAN" => Some(1),
        "FEB" => Some(2),
        "MAR" => Some(3),
        "APR" => Some(4),
        "MAY" => Some(5),
        "JUN" => Some(6),
        "JUL" => Some(7),
        "AUG" => Some(8),
        "SEP" => Some(9),
        "OCT" => Some(10),
        "NOV" => Some(11),
        "DEC" => Some(12),
        _ => None,
    }
}

fn parse_weekday(value: &str) -> Option<u32> {
    match value.to_uppercase().as_str() {
        "SUN" => Some(0),
        "MON" => Some(1),
        "TUE" => Some(2),
        "WED" => Some(3),
        "THU" => Some(4),
        "FRI" => Some(5),
        "SAT" => Some(6),
        _ => None,
    }
}

fn parse_field(input: &str) -> Result<Field, CronError> {
    if input == "*" {
        return Ok(Field::Any);
    }

    if input == "L" {
        return Ok(Field::LastDay);
    }

    if let Some(day) = input.strip_suffix('W') {
        return Ok(Field::NearestWeekday(
            day.parse().unwrap(),
        ));
    }

    if let Some((day, nth)) = input.split_once('#') {
        return Ok(Field::NthWeekday(
            parse_weekday(day).unwrap(),
            nth.parse().unwrap(),
        ));
    }

    if input.contains(',') {
        return Ok(Field::List(
            input
                .split(',')
                .map(parse_field)
                .collect::<Result<Vec<_>, _>>()?,
        ));
    }

    if let Some(month) = parse_month(input) {
        return Ok(Field::Value(month));
    }

    if let Some(day) = parse_weekday(input) {
        return Ok(Field::Value(day));
    }

    if let Some((range, step)) = input.split_once('/') {
        let step: u32 = step.parse().unwrap();

        if range == "*" {
            return Ok(Field::Step(step));
        }

        if let Some((a, b)) = range.split_once('-') {
            return Ok(Field::RangeStep(
                a.parse().unwrap(),
                b.parse().unwrap(),
                step,
            ));
        }
    }

    if let Some((a, b)) = input.split_once('-') {
        return Ok(Field::Range(
            a.parse().unwrap(),
            b.parse().unwrap(),
        ));
    }

    Ok(Field::Value(
        u32::from_str(input).unwrap(),
    ))
}


fn nearest_weekday(
    year: i32,
    month: u32,
    day: u32,
) -> u32 {
    use chrono::{Datelike, NaiveDate};

    let date =
        NaiveDate::from_ymd_opt(year, month, day)
            .unwrap();

    match date.weekday() {
        chrono::Weekday::Sat => {
            if day > 1 {
                day - 1
            } else {
                day + 2
            }
        }

        chrono::Weekday::Sun => {
            day + 1
        }

        _ => day,
    }
}

fn is_nth_weekday_of_month<Tz: TimeZone>(
    dt: DateTime<Tz>,
    weekday: u32,
    nth: u32,
) -> bool {
    let current_weekday =
        dt.weekday().num_days_from_sunday();

    if current_weekday != weekday {
        return false;
    }

    let occurrence =
        ((dt.day() - 1) / 7) + 1;

    occurrence == nth
}
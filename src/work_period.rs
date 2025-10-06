use chrono::{Datelike, NaiveDate};
use regex::Regex;

/// Processes work period markers in markdown and replaces them with calculated durations.
///
/// Syntax: `{{work_period: start="YYYY-MM", end="present"}}` or `{{work_period: start="YYYY-MM", end="YYYY-MM"}}`
///
/// Example:
/// - `{{work_period: start="2022-12", end="present"}}` → "2 years, 10 months"
/// - `{{work_period: start="2018-07", end="2021-11"}}` → "3 years, 5 months"
pub fn process(markdown: &str) -> String {
    let re = Regex::new(r#"\{\{work_period:\s*start="([^"]+)",?\s*end="([^"]+)"\}\}"#)
        .expect("Invalid regex");

    re.replace_all(markdown, |caps: &regex::Captures| {
        let start = &caps[1];
        let end = &caps[2];

        match calculate_duration(start, end) {
            Ok(duration) => duration,
            Err(e) => {
                eprintln!("Warning: Failed to parse work period (start={start}, end={end}): {e}");
                format!("{{{{work_period: start=\"{start}\", end=\"{end}\"}}}}")
            }
        }
    })
    .to_string()
}

/// Calculates duration between two dates and returns human-readable format.
///
/// If `end` is "present", uses current date.
fn calculate_duration(start: &str, end: &str) -> Result<String, Box<dyn std::error::Error>> {
    let start_date = parse_year_month(start)?;
    let end_date = if end.to_lowercase() == "present" {
        chrono::Local::now().date_naive()
    } else {
        parse_year_month(end)?
    };

    let (years, months) = months_between(start_date, end_date);
    Ok(format_duration(years, months))
}

/// Parses "YYYY-MM" string into NaiveDate (first day of the month).
fn parse_year_month(date_str: &str) -> Result<NaiveDate, Box<dyn std::error::Error>> {
    let parts: Vec<&str> = date_str.split('-').collect();
    if parts.len() != 2 {
        return Err(format!("Invalid date format: {date_str}. Expected YYYY-MM").into());
    }

    let year: i32 = parts[0].parse()?;
    let month: u32 = parts[1].parse()?;

    NaiveDate::from_ymd_opt(year, month, 1)
        .ok_or_else(|| format!("Invalid date: {date_str}").into())
}

/// Calculates years and months between two dates.
fn months_between(start: NaiveDate, end: NaiveDate) -> (i32, i32) {
    let mut years = end.year() - start.year();
    let mut months = end.month() as i32 - start.month() as i32;

    if months < 0 {
        years -= 1;
        months += 12;
    }

    // Adjust for day of month (if end day < start day, subtract one month)
    if end.day() < start.day() {
        months -= 1;
        if months < 0 {
            years -= 1;
            months += 12;
        }
    }

    (years, months)
}

/// Formats duration as "X years, Y months" with proper singular/plural handling.
fn format_duration(years: i32, months: i32) -> String {
    match (years, months) {
        (0, 0) => "0 months".to_string(),
        (0, m) => format!("{} {}", m, if m == 1 { "month" } else { "months" }),
        (y, 0) => format!("{} {}", y, if y == 1 { "year" } else { "years" }),
        (y, m) => format!(
            "{} {}, {} {}",
            y,
            if y == 1 { "year" } else { "years" },
            m,
            if m == 1 { "month" } else { "months" }
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_year_month() {
        let date = parse_year_month("2022-12").unwrap();
        assert_eq!(date.year(), 2022);
        assert_eq!(date.month(), 12);
        assert_eq!(date.day(), 1);
    }

    #[test]
    fn test_months_between() {
        let start = NaiveDate::from_ymd_opt(2022, 12, 1).unwrap();
        let end = NaiveDate::from_ymd_opt(2025, 10, 1).unwrap();
        let (years, months) = months_between(start, end);
        assert_eq!(years, 2);
        assert_eq!(months, 10);
    }

    #[test]
    fn test_format_duration() {
        assert_eq!(format_duration(0, 0), "0 months");
        assert_eq!(format_duration(0, 1), "1 month");
        assert_eq!(format_duration(0, 5), "5 months");
        assert_eq!(format_duration(1, 0), "1 year");
        assert_eq!(format_duration(2, 0), "2 years");
        assert_eq!(format_duration(2, 10), "2 years, 10 months");
        assert_eq!(format_duration(1, 1), "1 year, 1 month");
    }

    #[test]
    fn test_process() {
        let input = r#"Started {{work_period: start="2022-12", end="2023-03"}} ago"#;
        let output = process(input);
        assert_eq!(output, "Started 3 months ago");
    }
}

#![warn(clippy::all, clippy::pedantic)]
use chrono::{Datelike, NaiveDate};
use regex::Regex;
use std::convert::TryFrom;
use std::fs;

/// Processes work period markers in markdown and replaces them with calculated durations.
///
/// Syntax:
/// - `{{work_period: start="YYYY-MM", end="present"}}` → "2 years, 10 months"
/// - `{{work_period: start="YYYY-MM", end="YYYY-MM"}}` → "3 years, 5 months"
/// - `{{total_work_period}}` → sum of all `work_period` markers (reads from cv.md if needed)
///
/// Example:
/// ```
/// {{work_period: start="2022-12", end="present"}}
/// {{work_period: start="2018-07", end="2021-11"}}
/// Total: {{total_work_period}}
/// ```
pub fn process(markdown: &str) -> String {
    let mut durations = Vec::new();

    // First pass: replace individual work_period markers and collect durations
    let re = Regex::new(r#"\{\{work_period:\s*start="([^"]+)",?\s*end="([^"]+)"\}\}"#)
        .expect("Invalid regex");

    let after_work_periods = re
        .replace_all(markdown, |caps: &regex::Captures| {
            let start = &caps[1];
            let end = &caps[2];

            match calculate_duration_parts(start, end) {
                Ok((years, months)) => {
                    durations.push((years, months));
                    format_duration(years, months)
                }
                Err(e) => {
                    eprintln!(
                        "Warning: Failed to parse work period (start={start}, end={end}): {e}"
                    );
                    format!("{{{{work_period: start=\"{start}\", end=\"{end}\"}}}}")
                }
            }
        })
        .to_string();

    // Second pass: replace total_work_period with sum (years only, rounded)
    // If no work periods found in current file but total_work_period exists, read from cv.md
    if durations.is_empty() && after_work_periods.contains("{{total_work_period}}") {
        durations = extract_durations_from_cv();
    }

    let total = sum_durations(&durations);
    after_work_periods.replace(
        "{{total_work_period}}",
        &format_duration_years_only(total.0, total.1),
    )
}

/// Extracts work period durations from cv.md file.
fn extract_durations_from_cv() -> Vec<(i32, i32)> {
    let cv_path = "in/cv.md";
    match fs::read_to_string(cv_path) {
        Ok(cv_content) => {
            let mut durations = Vec::new();
            let re = Regex::new(r#"\{\{work_period:\s*start="([^"]+)",?\s*end="([^"]+)"\}\}"#)
                .expect("Invalid regex");

            for caps in re.captures_iter(&cv_content) {
                let start = &caps[1];
                let end = &caps[2];

                if let Ok((years, months)) = calculate_duration_parts(start, end) {
                    durations.push((years, months));
                }
            }
            durations
        }
        Err(e) => {
            eprintln!("Warning: Failed to read cv.md for total_work_period: {e}");
            Vec::new()
        }
    }
}

/// Calculates duration between two dates and returns (years, months).
///
/// If `end` is "present", uses current date.
fn calculate_duration_parts(
    start: &str,
    end: &str,
) -> Result<(i32, i32), Box<dyn std::error::Error>> {
    let start_date = parse_year_month(start)?;
    let end_date = if end.to_lowercase() == "present" {
        chrono::Local::now().date_naive()
    } else {
        parse_year_month(end)?
    };

    Ok(months_between(start_date, end_date))
}

/// Sums multiple durations (years, months) into a single total duration.
fn sum_durations(durations: &[(i32, i32)]) -> (i32, i32) {
    let total_months: i32 = durations.iter().map(|(y, m)| y * 12 + m).sum();
    let years = total_months / 12;
    let months = total_months % 12;
    (years, months)
}

/// Parses "YYYY-MM" string into `NaiveDate` (first day of the month).
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
    let end_month = i32::try_from(end.month()).expect("month fits in i32");
    let start_month = i32::try_from(start.month()).expect("month fits in i32");
    let mut months = end_month - start_month;

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

/// Formats duration as years only, rounding up if months >= 6.
fn format_duration_years_only(years: i32, months: i32) -> String {
    let rounded_years = if months >= 6 { years + 1 } else { years };
    format!(
        "{} {}",
        rounded_years,
        if rounded_years == 1 { "year" } else { "years" }
    )
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

    #[test]
    fn test_sum_durations() {
        let durations = vec![(2, 10), (3, 5), (2, 2)];
        let (years, months) = sum_durations(&durations);
        // 2*12+10 + 3*12+5 + 2*2+2 = 34 + 41 + 26 = 101 months = 8 years, 5 months
        assert_eq!(years, 8);
        assert_eq!(months, 5);
    }

    #[test]
    fn test_format_duration_years_only() {
        assert_eq!(format_duration_years_only(9, 2), "9 years"); // 9y2m rounds down to 9y
        assert_eq!(format_duration_years_only(9, 6), "10 years"); // 9y6m rounds up to 10y
        assert_eq!(format_duration_years_only(9, 11), "10 years"); // 9y11m rounds up to 10y
        assert_eq!(format_duration_years_only(1, 0), "1 year"); // singular
        assert_eq!(format_duration_years_only(0, 5), "0 years"); // less than 6 months = 0 years
        assert_eq!(format_duration_years_only(0, 9), "1 year"); // 9 months rounds to 1 year
    }

    #[test]
    fn test_total_work_period() {
        let input = r#"
Exp 1: {{work_period: start="2022-12", end="2023-03"}}
Exp 2: {{work_period: start="2020-01", end="2020-07"}}
Total: {{total_work_period}}
"#;
        let output = process(input);
        // 3 months + 6 months = 9 months, rounds up to 1 year
        assert!(output.contains("Total: 1 year"));
    }
}

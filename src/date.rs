use crate::model::FileDateType;
use chrono::{DateTime, Datelike, Utc};
use color_eyre::eyre::{Context, ContextCompat, Result};
use std::fs;
use std::path::Path;

struct FileTimestamps {
    created: DateTime<Utc>,
    modified: DateTime<Utc>,
    accessed: DateTime<Utc>,
}

/// Get the most recent timestamp based on selected file date types
pub fn get_file_date(path: &Path, date_types: &[FileDateType]) -> Result<DateTime<Utc>> {
    let file_timestamps = get_file_timestamps(path)?;
    let created = file_timestamps.created;
    let modified = file_timestamps.modified;
    let accessed = file_timestamps.accessed;

    let timestamps = date_types.iter()
        .map(|t| match t {
            FileDateType::Created => created,
            FileDateType::Modified => modified,
            FileDateType::Accessed => accessed,
        })
        .max();

    timestamps.context("At least one file date type must be provided")
}

fn get_file_timestamps(path: &Path) -> Result<FileTimestamps> {
    let metadata = fs::metadata(path)
        .with_context(|| format!("Failed to get metadata for: {}", path.display()))?;

    let created = metadata.created()
        .with_context(|| format!("Failed to get creation time for: {}", path.display()))?;
    let modified = metadata.modified()
        .with_context(|| format!("Failed to get modified time for: {}", path.display()))?;
    let accessed = metadata.accessed()
        .with_context(|| format!("Failed to get accessed time for: {}", path.display()))?;

    Ok(FileTimestamps {
        created: created.into(),
        modified: modified.into(),
        accessed: accessed.into(),
    })
}

/// Get the current week identifier (for comparison)
pub fn get_current_week(now: DateTime<Utc>) -> (i32, u32) {
    let iso_week = now.iso_week();
    (iso_week.year(), iso_week.week())
}

/// Get the current month identifier (for comparison)
pub fn get_current_month(now: DateTime<Utc>) -> (i32, u32) {
    (now.year(), now.month())
}

/// Get the current year
pub fn get_current_year(now: DateTime<Utc>) -> i32 {
    now.year()
}

/// Get the current semester identifier (for comparison)
pub fn get_current_semester(now: DateTime<Utc>) -> (i32, u32) {
    let semester = calculate_semester(now.month());
    (now.year(), semester)
}

/// Get the current trimester identifier (for comparison)
pub fn get_current_trimester(now: DateTime<Utc>) -> (i32, u32) {
    let trimester = calculate_trimester(now.month());
    (now.year(), trimester)
}

/// Get the current quadrimester identifier (for comparison)
pub fn get_current_quadrimester(now: DateTime<Utc>) -> (i32, u32) {
    let quadrimester = calculate_quadrimester(now.month());
    (now.year(), quadrimester)
}

/// Get the current biweekly identifier (for comparison)
pub fn get_current_biweekly(now: DateTime<Utc>) -> (i32, u32) {
    let iso_week = now.iso_week();
    let biweekly = calculate_biweekly(iso_week.week());
    (iso_week.year(), biweekly)
}

/// Check if a date is before the current week
pub fn is_before_current_week(date: DateTime<Utc>, now: DateTime<Utc>) -> bool {
    let current = get_current_week(now);
    let file_week = date.iso_week();
    let file_identifier = (file_week.year(), file_week.week());

    file_identifier < current
}

/// Check if a date is before the current month
pub fn is_before_current_month(date: DateTime<Utc>, now: DateTime<Utc>) -> bool {
    let current = get_current_month(now);
    let file_identifier = (date.year(), date.month());

    file_identifier < current
}

/// Check if a date is before the current year
pub fn is_before_current_year(date: DateTime<Utc>, now: DateTime<Utc>) -> bool {
    date.year() < get_current_year(now)
}

/// Check if a date is before the current semester
pub fn is_before_current_semester(date: DateTime<Utc>, now: DateTime<Utc>) -> bool {
    let current = get_current_semester(now);
    let semester = calculate_semester(date.month());
    let file_identifier = (date.year(), semester);

    file_identifier < current
}

/// Check if a date is before the current trimester
pub fn is_before_current_trimester(date: DateTime<Utc>, now: DateTime<Utc>) -> bool {
    let current = get_current_trimester(now);
    let trimester = calculate_trimester(date.month());
    let file_identifier = (date.year(), trimester);

    file_identifier < current
}

/// Check if a date is before the current quadrimester
pub fn is_before_current_quadrimester(date: DateTime<Utc>, now: DateTime<Utc>) -> bool {
    let current = get_current_quadrimester(now);
    let quadrimester = calculate_quadrimester(date.month());
    let file_identifier = (date.year(), quadrimester);

    file_identifier < current
}

/// Check if a date is before the current biweekly period
pub fn is_before_current_biweekly(date: DateTime<Utc>, now: DateTime<Utc>) -> bool {
    let current = get_current_biweekly(now);
    let iso_week = date.iso_week();
    let biweekly = calculate_biweekly(iso_week.week());
    let file_identifier = (iso_week.year(), biweekly);

    file_identifier < current
}

/// Get the week identifier string (e.g., "2025-W49")
pub fn get_week_identifier(date: DateTime<Utc>) -> String {
    let iso_week = date.iso_week();
    format!("{}-W{:02}", iso_week.year(), iso_week.week())
}

/// Get the month identifier string (e.g., "2025-11")
pub fn get_month_identifier(date: DateTime<Utc>) -> String {
    format!("{}-{:02}", date.year(), date.month())
}

/// Calculate semester number (1 or 2) from month
pub fn calculate_semester(month: u32) -> u32 {
    validate_month(month);
    if month <= 6 { 1 } else { 2 }
}

/// Calculate trimester number (1-4) from month
pub fn calculate_trimester(month: u32) -> u32 {
    validate_month(month);
    (month - 1) / 3 + 1
}

/// Calculate quadrimester number (1-3) from month
pub fn calculate_quadrimester(month: u32) -> u32 {
    validate_month(month);
    (month - 1) / 4 + 1
}

fn validate_month(month: u32) {
    debug_assert!(month >= 1 && month <= 12, "month must be between 1 and 12, got {}", month);
}

/// Calculate biweekly number from ISO week (handles week 53 edge case)
pub fn calculate_biweekly(iso_week: u32) -> u32 {
    debug_assert!(iso_week > 0 && iso_week <= 53, "iso_week must be between 1 and 53, got {}", iso_week);
    // Weeks 51-53 all map to BW26
    if iso_week >= 51 {
        26
    } else {
        (iso_week - 1) / 2 + 1
    }
}

/// Get the year identifier string (e.g., "2025")
pub fn get_year_identifier(date: DateTime<Utc>) -> String {
    format!("{}", date.year())
}

/// Get the semester identifier string (e.g., "2025-H1")
pub fn get_semester_identifier(date: DateTime<Utc>) -> String {
    let semester = calculate_semester(date.month());
    format!("{}-H{}", date.year(), semester)
}

/// Get the trimester identifier string (e.g., "2025-Q1")
pub fn get_trimester_identifier(date: DateTime<Utc>) -> String {
    let trimester = calculate_trimester(date.month());
    format!("{}-Q{}", date.year(), trimester)
}

/// Get the quadrimester identifier string (e.g., "2025-QD1")
pub fn get_quadrimester_identifier(date: DateTime<Utc>) -> String {
    let quadrimester = calculate_quadrimester(date.month());
    format!("{}-QD{}", date.year(), quadrimester)
}

/// Get the biweekly identifier string (e.g., "2025-BW01")
pub fn get_biweekly_identifier(date: DateTime<Utc>) -> String {
    let iso_week = date.iso_week();
    let biweekly = calculate_biweekly(iso_week.week());
    format!("{}-BW{:02}", iso_week.year(), biweekly)
}

#[cfg(test)]
mod tests {
    use super::*;

    // Period calculation tests
    #[test]
    fn test_calculate_semester() {
        // First semester: January through June
        assert_eq!(calculate_semester(1), 1);
        assert_eq!(calculate_semester(2), 1);
        assert_eq!(calculate_semester(3), 1);
        assert_eq!(calculate_semester(4), 1);
        assert_eq!(calculate_semester(5), 1);
        assert_eq!(calculate_semester(6), 1);

        // Second semester: July through December
        assert_eq!(calculate_semester(7), 2);
        assert_eq!(calculate_semester(8), 2);
        assert_eq!(calculate_semester(9), 2);
        assert_eq!(calculate_semester(10), 2);
        assert_eq!(calculate_semester(11), 2);
        assert_eq!(calculate_semester(12), 2);
    }

    #[test]
    fn test_calculate_trimester() {
        // Q1: January through March
        assert_eq!(calculate_trimester(1), 1);
        assert_eq!(calculate_trimester(2), 1);
        assert_eq!(calculate_trimester(3), 1);

        // Q2: April through June
        assert_eq!(calculate_trimester(4), 2);
        assert_eq!(calculate_trimester(5), 2);
        assert_eq!(calculate_trimester(6), 2);

        // Q3: July through September
        assert_eq!(calculate_trimester(7), 3);
        assert_eq!(calculate_trimester(8), 3);
        assert_eq!(calculate_trimester(9), 3);

        // Q4: October through December
        assert_eq!(calculate_trimester(10), 4);
        assert_eq!(calculate_trimester(11), 4);
        assert_eq!(calculate_trimester(12), 4);

        let result = std::panic::catch_unwind(|| {
            calculate_trimester(0)
        });
        assert!(result.is_err(), "Expected panic for invalid month 0");

        let result = std::panic::catch_unwind(|| {
            calculate_trimester(13)
        });
        assert!(result.is_err(), "Expected panic for invalid month 13");
    }

    #[test]
    fn test_calculate_quadrimester() {
        // QD1: January through April
        assert_eq!(calculate_quadrimester(1), 1);
        assert_eq!(calculate_quadrimester(2), 1);
        assert_eq!(calculate_quadrimester(3), 1);
        assert_eq!(calculate_quadrimester(4), 1);

        // QD2: May through August
        assert_eq!(calculate_quadrimester(5), 2);
        assert_eq!(calculate_quadrimester(6), 2);
        assert_eq!(calculate_quadrimester(7), 2);
        assert_eq!(calculate_quadrimester(8), 2);

        // QD3: September through December
        assert_eq!(calculate_quadrimester(9), 3);
        assert_eq!(calculate_quadrimester(10), 3);
        assert_eq!(calculate_quadrimester(11), 3);
        assert_eq!(calculate_quadrimester(12), 3);

        let result = std::panic::catch_unwind(|| {
            calculate_quadrimester(0)
        });
        assert!(result.is_err(), "Expected panic for invalid month 0");

        let result = std::panic::catch_unwind(|| {
            calculate_quadrimester(13)
        });
        assert!(result.is_err(), "Expected panic for invalid month 13");
    }

    #[test]
    fn test_calculate_biweekly() {
        // Normal weeks: weeks 1-50
        assert_eq!(calculate_biweekly(1), 1);  // Week 1 -> BW1
        assert_eq!(calculate_biweekly(2), 1);  // Week 2 -> BW1
        assert_eq!(calculate_biweekly(3), 2);  // Week 3 -> BW2
        assert_eq!(calculate_biweekly(4), 2);  // Week 4 -> BW2
        assert_eq!(calculate_biweekly(5), 3);  // Week 5 -> BW3
        assert_eq!(calculate_biweekly(10), 5); // Week 10 -> BW5
        assert_eq!(calculate_biweekly(25), 13); // Week 25 -> BW13
        assert_eq!(calculate_biweekly(49), 25); // Week 49 -> BW25
        assert_eq!(calculate_biweekly(50), 25); // Week 50 -> BW25

        // Edge case: weeks 51-53 all map to BW26
        assert_eq!(calculate_biweekly(51), 26);
        assert_eq!(calculate_biweekly(52), 26);
        assert_eq!(calculate_biweekly(53), 26);

        let result = std::panic::catch_unwind(|| {
            calculate_biweekly(0)
        });
        assert!(result.is_err(), "Expected panic for invalid week 0");

        let result = std::panic::catch_unwind(|| {
            calculate_biweekly(54)
        });
        assert!(result.is_err(), "Expected panic for invalid week 54");
    }

    // Identifier formatting tests
    #[test]
    fn test_get_week_identifier() {
        // Week 1 (with zero padding)
        let date = "2025-01-06T00:00:00Z".parse::<DateTime<Utc>>().unwrap();
        assert_eq!(get_week_identifier(date), "2025-W02");

        // Week 10
        let date = "2025-03-10T00:00:00Z".parse::<DateTime<Utc>>().unwrap();
        assert_eq!(get_week_identifier(date), "2025-W11");

        // Week 52
        let date = "2025-12-29T00:00:00Z".parse::<DateTime<Utc>>().unwrap();
        assert_eq!(get_week_identifier(date), "2026-W01");

        // Year boundary: December 29, 2024 is in week 1 of 2025 (ISO week)
        let date = "2024-12-30T00:00:00Z".parse::<DateTime<Utc>>().unwrap();
        assert_eq!(get_week_identifier(date), "2025-W01");
    }

    #[test]
    fn test_get_month_identifier() {
        // January (with zero padding)
        let date = "2025-01-15T00:00:00Z".parse::<DateTime<Utc>>().unwrap();
        assert_eq!(get_month_identifier(date), "2025-01");

        // December
        let date = "2025-12-31T00:00:00Z".parse::<DateTime<Utc>>().unwrap();
        assert_eq!(get_month_identifier(date), "2025-12");

        // October (double digit)
        let date = "2025-10-01T00:00:00Z".parse::<DateTime<Utc>>().unwrap();
        assert_eq!(get_month_identifier(date), "2025-10");
    }

    #[test]
    fn test_get_year_identifier() {
        let date = "2025-06-15T00:00:00Z".parse::<DateTime<Utc>>().unwrap();
        assert_eq!(get_year_identifier(date), "2025");

        let date = "2024-01-01T00:00:00Z".parse::<DateTime<Utc>>().unwrap();
        assert_eq!(get_year_identifier(date), "2024");
    }

    #[test]
    fn test_get_semester_identifier() {
        // First semester (January)
        let date = "2025-01-15T00:00:00Z".parse::<DateTime<Utc>>().unwrap();
        assert_eq!(get_semester_identifier(date), "2025-H1");

        // First semester (June)
        let date = "2025-06-30T00:00:00Z".parse::<DateTime<Utc>>().unwrap();
        assert_eq!(get_semester_identifier(date), "2025-H1");

        // Second semester (July)
        let date = "2025-07-01T00:00:00Z".parse::<DateTime<Utc>>().unwrap();
        assert_eq!(get_semester_identifier(date), "2025-H2");

        // Second semester (December)
        let date = "2025-12-31T00:00:00Z".parse::<DateTime<Utc>>().unwrap();
        assert_eq!(get_semester_identifier(date), "2025-H2");
    }

    #[test]
    fn test_get_trimester_identifier() {
        // Q1
        let date = "2025-02-15T00:00:00Z".parse::<DateTime<Utc>>().unwrap();
        assert_eq!(get_trimester_identifier(date), "2025-Q1");

        // Q2
        let date = "2025-05-01T00:00:00Z".parse::<DateTime<Utc>>().unwrap();
        assert_eq!(get_trimester_identifier(date), "2025-Q2");

        // Q3
        let date = "2025-08-20T00:00:00Z".parse::<DateTime<Utc>>().unwrap();
        assert_eq!(get_trimester_identifier(date), "2025-Q3");

        // Q4
        let date = "2025-11-30T00:00:00Z".parse::<DateTime<Utc>>().unwrap();
        assert_eq!(get_trimester_identifier(date), "2025-Q4");
    }

    #[test]
    fn test_get_quadrimester_identifier() {
        // QD1
        let date = "2025-03-15T00:00:00Z".parse::<DateTime<Utc>>().unwrap();
        assert_eq!(get_quadrimester_identifier(date), "2025-QD1");

        // QD2
        let date = "2025-07-01T00:00:00Z".parse::<DateTime<Utc>>().unwrap();
        assert_eq!(get_quadrimester_identifier(date), "2025-QD2");

        // QD3
        let date = "2025-10-20T00:00:00Z".parse::<DateTime<Utc>>().unwrap();
        assert_eq!(get_quadrimester_identifier(date), "2025-QD3");
    }

    #[test]
    fn test_get_biweekly_identifier() {
        // BW01 (with zero padding)
        let date = "2025-01-06T00:00:00Z".parse::<DateTime<Utc>>().unwrap();
        assert_eq!(get_biweekly_identifier(date), "2025-BW01");

        // BW13 (mid-year)
        let date = "2025-06-23T00:00:00Z".parse::<DateTime<Utc>>().unwrap();
        assert_eq!(get_biweekly_identifier(date), "2025-BW13");

        // BW26 (week 52 edge case)
        let date = "2024-12-26T00:00:00Z".parse::<DateTime<Utc>>().unwrap();
        assert_eq!(get_biweekly_identifier(date), "2024-BW26");
    }

    // Time comparison tests
    #[test]
    fn test_is_before_current_week() {
        let now = "2025-06-15T00:00:00Z".parse::<DateTime<Utc>>().unwrap(); // Week 24

        // Same week - should return false
        let same_week = "2025-06-16T00:00:00Z".parse::<DateTime<Utc>>().unwrap();
        assert!(!is_before_current_week(same_week, now));

        // Previous week - should return true
        let previous_week = "2025-06-08T00:00:00Z".parse::<DateTime<Utc>>().unwrap(); // Week 23
        assert!(is_before_current_week(previous_week, now));

        // Next week - should return false
        let next_week = "2025-06-22T00:00:00Z".parse::<DateTime<Utc>>().unwrap(); // Week 25
        assert!(!is_before_current_week(next_week, now));

        // Year boundary: week from previous year
        let previous_year = "2024-12-25T00:00:00Z".parse::<DateTime<Utc>>().unwrap();
        assert!(is_before_current_week(previous_year, now));

        // Far past
        let far_past = "2024-01-01T00:00:00Z".parse::<DateTime<Utc>>().unwrap();
        assert!(is_before_current_week(far_past, now));

        // Far future
        let far_future = "2026-01-01T00:00:00Z".parse::<DateTime<Utc>>().unwrap();
        assert!(!is_before_current_week(far_future, now));
    }

    #[test]
    fn test_is_before_current_month() {
        let now = "2025-06-15T00:00:00Z".parse::<DateTime<Utc>>().unwrap(); // June 2025

        // Same month - should return false
        let same_month = "2025-06-01T00:00:00Z".parse::<DateTime<Utc>>().unwrap();
        assert!(!is_before_current_month(same_month, now));

        // Previous month - should return true
        let previous_month = "2025-05-31T00:00:00Z".parse::<DateTime<Utc>>().unwrap();
        assert!(is_before_current_month(previous_month, now));

        // Next month - should return false
        let next_month = "2025-07-01T00:00:00Z".parse::<DateTime<Utc>>().unwrap();
        assert!(!is_before_current_month(next_month, now));

        // Year boundary: December of previous year
        let previous_year = "2024-12-31T00:00:00Z".parse::<DateTime<Utc>>().unwrap();
        assert!(is_before_current_month(previous_year, now));

        // Year boundary: January of next year
        let next_year = "2026-01-01T00:00:00Z".parse::<DateTime<Utc>>().unwrap();
        assert!(!is_before_current_month(next_year, now));
    }

    #[test]
    fn test_is_before_current_year() {
        let now = "2025-06-15T00:00:00Z".parse::<DateTime<Utc>>().unwrap(); // 2025

        // Same year - should return false
        let same_year = "2025-01-01T00:00:00Z".parse::<DateTime<Utc>>().unwrap();
        assert!(!is_before_current_year(same_year, now));

        let same_year_end = "2025-12-31T00:00:00Z".parse::<DateTime<Utc>>().unwrap();
        assert!(!is_before_current_year(same_year_end, now));

        // Previous year - should return true
        let previous_year = "2024-12-31T00:00:00Z".parse::<DateTime<Utc>>().unwrap();
        assert!(is_before_current_year(previous_year, now));

        // Next year - should return false
        let next_year = "2026-01-01T00:00:00Z".parse::<DateTime<Utc>>().unwrap();
        assert!(!is_before_current_year(next_year, now));
    }

    #[test]
    fn test_is_before_current_semester() {
        // Test with now in H1 (January)
        let now_h1 = "2025-03-15T00:00:00Z".parse::<DateTime<Utc>>().unwrap(); // H1

        let same_semester = "2025-01-01T00:00:00Z".parse::<DateTime<Utc>>().unwrap();
        assert!(!is_before_current_semester(same_semester, now_h1));

        let previous_semester = "2024-12-31T00:00:00Z".parse::<DateTime<Utc>>().unwrap(); // 2024-H2
        assert!(is_before_current_semester(previous_semester, now_h1));

        let next_semester = "2025-07-01T00:00:00Z".parse::<DateTime<Utc>>().unwrap(); // 2025-H2
        assert!(!is_before_current_semester(next_semester, now_h1));

        // Test with now in H2 (August)
        let now_h2 = "2025-08-15T00:00:00Z".parse::<DateTime<Utc>>().unwrap(); // H2

        let previous_semester_h2 = "2025-06-30T00:00:00Z".parse::<DateTime<Utc>>().unwrap(); // 2025-H1
        assert!(is_before_current_semester(previous_semester_h2, now_h2));

        let same_semester_h2 = "2025-12-31T00:00:00Z".parse::<DateTime<Utc>>().unwrap(); // 2025-H2
        assert!(!is_before_current_semester(same_semester_h2, now_h2));
    }

    #[test]
    fn test_is_before_current_trimester() {
        let now = "2025-05-15T00:00:00Z".parse::<DateTime<Utc>>().unwrap(); // Q2

        // Same trimester
        let same_trimester = "2025-04-01T00:00:00Z".parse::<DateTime<Utc>>().unwrap();
        assert!(!is_before_current_trimester(same_trimester, now));

        // Previous trimester
        let previous_trimester = "2025-03-31T00:00:00Z".parse::<DateTime<Utc>>().unwrap(); // Q1
        assert!(is_before_current_trimester(previous_trimester, now));

        // Next trimester
        let next_trimester = "2025-07-01T00:00:00Z".parse::<DateTime<Utc>>().unwrap(); // Q3
        assert!(!is_before_current_trimester(next_trimester, now));

        // Previous year
        let previous_year = "2024-05-15T00:00:00Z".parse::<DateTime<Utc>>().unwrap();
        assert!(is_before_current_trimester(previous_year, now));
    }

    #[test]
    fn test_is_before_current_quadrimester() {
        let now = "2025-06-15T00:00:00Z".parse::<DateTime<Utc>>().unwrap(); // QD2

        // Same quadrimester
        let same_qd = "2025-05-01T00:00:00Z".parse::<DateTime<Utc>>().unwrap();
        assert!(!is_before_current_quadrimester(same_qd, now));

        // Previous quadrimester
        let previous_qd = "2025-04-30T00:00:00Z".parse::<DateTime<Utc>>().unwrap(); // QD1
        assert!(is_before_current_quadrimester(previous_qd, now));

        // Next quadrimester
        let next_qd = "2025-09-01T00:00:00Z".parse::<DateTime<Utc>>().unwrap(); // QD3
        assert!(!is_before_current_quadrimester(next_qd, now));

        // Previous year
        let previous_year = "2024-06-15T00:00:00Z".parse::<DateTime<Utc>>().unwrap();
        assert!(is_before_current_quadrimester(previous_year, now));
    }

    #[test]
    fn test_is_before_current_biweekly() {
        let now = "2025-06-15T00:00:00Z".parse::<DateTime<Utc>>().unwrap(); // Week 24 -> BW12

        // Same biweekly period
        let same_biweekly = "2025-06-16T00:00:00Z".parse::<DateTime<Utc>>().unwrap();
        assert!(!is_before_current_biweekly(same_biweekly, now));

        // Previous biweekly period
        let previous_biweekly = "2025-06-01T00:00:00Z".parse::<DateTime<Utc>>().unwrap(); // Week 22 -> BW11
        assert!(is_before_current_biweekly(previous_biweekly, now));

        // Next biweekly period
        let next_biweekly = "2025-06-30T00:00:00Z".parse::<DateTime<Utc>>().unwrap(); // Week 27 -> BW14
        assert!(!is_before_current_biweekly(next_biweekly, now));

        // Year boundary
        let previous_year = "2024-12-25T00:00:00Z".parse::<DateTime<Utc>>().unwrap();
        assert!(is_before_current_biweekly(previous_year, now));
    }
}

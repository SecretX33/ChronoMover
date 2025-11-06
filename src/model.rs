use crate::log;
use chrono::{DateTime, Local, NaiveDate, NaiveDateTime, Offset, Utc};
use clap::{Parser, ValueEnum};
use color_eyre::eyre;
use color_eyre::eyre::{bail, Context};
use std::fs;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    #[arg(short, long, required = true, value_name = "PATH", help = "Source directory containing files to organize")]
    pub source: PathBuf,

    #[arg(short, long, required = true, value_name = "PATH", help = "Destination directory where files will be moved")]
    pub destination: PathBuf,

    #[arg(short, long, value_enum, value_name = "STRATEGY", help = "Optional grouping strategy")]
    pub group_by: Option<GroupBy>,

    #[arg(long, default_value = "false", help = "Only move files from previous periods (not current period). Only valid with --group-by")]
    pub previous_period_only: bool,

    #[arg(long, value_name = "DURATION_OR_DATE", value_parser = parse_older_than, help = "Only move files older than specified duration or date (e.g., \"30d\", \"1y6M\", \"2025-01-15\", \"2025-01-15T06:30:53\")")]
    pub older_than: Option<DateTime<Utc>>,

    #[arg(
        long,
        default_value = "created,modified",
        value_delimiter = ',',
        value_parser = file_date_type_parser,
        value_name = "TYPES",
        help = "Which timestamps to check (created, modified, accessed). Can use short forms (c, m, a)"
    )]
    pub file_date_types: Vec<FileDateType>,

    #[arg(long, value_name = "PATHS", value_delimiter = ',', help = "Comma-separated list of files/folders to ignore (absolute paths)")]
    pub ignored_paths: Option<Vec<PathBuf>>,

    #[arg(long, value_name = "DEPTH", help = "Minimum directory depth to search")]
    pub min_depth: Option<usize>,

    #[arg(long, value_name = "DEPTH", help = "Maximum directory depth to search")]
    pub max_depth: Option<usize>,

    #[arg(long, default_value = "false", help = "Keep empty folders after moving files")]
    pub keep_empty_folders: bool,

    #[arg(long, default_value = "false", help = "Follow symbolic links while traversing")]
    pub follow_symbolic_links: bool,

    #[arg(long, default_value = "false", help = "Preview what would be moved without actually moving files")]
    pub dry_run: bool,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum GroupBy {
    /// Group by ISO week (e.g., 2025-49)
    Week,
    /// Group by biweekly period (e.g., 2025-BW01 through 2025-BW26)
    Biweekly,
    /// Group by month (e.g., 2025-11)
    Month,
    /// Group by trimester/quarter (e.g., 2025-Q1 through 2025-Q4)
    Trimester,
    /// Group by quadrimester (e.g., 2025-QD1 through 2025-QD3)
    Quadrimester,
    /// Group by semester/half-year (e.g., 2025-H1, 2025-H2)
    Semester,
    /// Group by year (e.g., 2025)
    Year,
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum FileDateType {
    Created,
    Modified,
    Accessed,
}

/// Parse file date type from string
fn file_date_type_parser(value: &str) -> color_eyre::Result<FileDateType, String> {
    let trimmed_value = value.trim();
    match trimmed_value.to_ascii_lowercase().as_str() {
        "c" | "created" => Ok(FileDateType::Created),
        "m" | "modified" => Ok(FileDateType::Modified),
        "a" | "accessed" => Ok(FileDateType::Accessed),
        _ => Err(format!(
            "Unsupported file date type: {}. Please use one of the following: {}",
            trimmed_value,
            ["created (c)", "modified (m)", "accessed (a)"].join(", ")
        )),
    }
}

/// Parse --older-than argument (duration or ISO date/datetime)
fn parse_older_than(value: &str) -> color_eyre::Result<DateTime<Utc>> {
    // Try parsing as ISO datetime first
    let iso_datetime_option =  NaiveDateTime::parse_from_str(value, "%Y-%m-%dT%H:%M:%S").ok()
        .and_then(|dt| {
            let local_offset =  Local::now().offset().fix();
            return dt.and_local_timezone(local_offset).single()
        })
        .map(|dt| dt.to_utc());

    if let Some(dt) = iso_datetime_option {
        return Ok(dt);
    }

    // Try parsing as ISO date
    let iso_date_option = NaiveDate::parse_from_str(value, "%Y-%m-%d").ok()
        .and_then(|date| date.and_hms_opt(0, 0, 0))
        .and_then(|dt| {
            let local_offset =  Local::now().offset().fix();
            return dt.and_local_timezone(local_offset).single()
        })
        .map(|dt| dt.to_utc());

    if let Some(dt) = iso_date_option {
        return Ok(dt);
    }

    // Try parsing as humantime duration
    if let Ok(duration) = humantime::parse_duration(value) {
        let now = Utc::now();
        let cutoff = now - duration;
        return Ok(cutoff);
    }

    Err(eyre::eyre!("Invalid format. Use duration (e.g., '30d', '1y6M'), ISO date ('2025-01-15'), or ISO datetime ('2025-01-15T10:30:00')"))
}

pub fn validate_arguments(args: &Args) -> color_eyre::Result<()> {
    if !args.source.exists() {
        bail!("Source directory does not exist: {}", args.source.display());
    }
    if !args.source.is_dir() {
        bail!("Source path is not a directory: {}", args.source.display());
    }

    if !args.destination.exists() {
        // Create destination directory if it doesn't exist
        log!("Destination directory does not exist. Creating: {}", args.destination.display());

        fs::create_dir_all(&args.destination)
            .with_context(|| format!("Failed to create destination directory: {}", args.destination.display()))?;
    }
    if !args.destination.is_dir() {
        bail!("Destination path is not a directory: {}", args.destination.display());
    }

    if args.source == args.destination {
        bail!("Source and destination directories cannot be the same");
    }

    if args.previous_period_only && args.group_by.is_none() {
        log!("WARNING: --previous-period-only is only meaningful with --group-by");
    }

    if let Some(ignored_paths) = &args.ignored_paths {
        for path in ignored_paths {
            if !path.exists() {
                log!("WARNING: Ignored path does not exist: {}", path.display());
            }
        }
    }

    if let (Some(min_depth), Some(max_depth)) = (args.min_depth, args.max_depth) {
        if min_depth > max_depth {
            bail!("Minimum depth ({}) must be less than or equal to maximum depth ({})", min_depth, max_depth);
        }
    }

    Ok(())
}

pub fn print_arguments(args: &Args) {
    log!("These are the arguments you provided:");
    log!("Source directory: {}", args.source.display());
    log!("Destination directory: {}", args.destination.display());
    log!("Finding files to move by their: {:?}", args.file_date_types);
    log!("Grouping By: {}", args.group_by.map(|e| format!("{:?}", e)).unwrap_or("None".to_string()));
    if args.previous_period_only {
        log!("Filter: Previous periods only (excluding current period)");
    }
    if let Some(cutoff) = args.older_than {
        log!("Filter: Only files older than {}", cutoff);
    }
    if let Some(ignored_paths) = &args.ignored_paths {
        log!("Ignored paths: {:?}", ignored_paths.iter().map(|p| p.display()).collect::<Vec<_>>());
    }
    if let Some(min_depth) = args.min_depth {
        log!("Min depth: {}", min_depth);
    }
    if let Some(max_depth) = args.max_depth {
        log!("Max depth: {}", max_depth);
    }
    if args.keep_empty_folders {
        log!("Keeping empty folders after moving files");
    }
    log!("Follow symbolic links: {}", args.follow_symbolic_links);
    log!("Dry run: {}", args.dry_run);
    log!("");
}

#[cfg(test)]
mod tests {
    use super::*;

    // file_date_type_parser tests
    #[test]
    fn test_file_date_type_parser_valid_full_names() {
        assert_eq!(file_date_type_parser("created").unwrap(), FileDateType::Created);
        assert_eq!(file_date_type_parser("modified").unwrap(), FileDateType::Modified);
        assert_eq!(file_date_type_parser("accessed").unwrap(), FileDateType::Accessed);
    }

    #[test]
    fn test_file_date_type_parser_valid_short_forms() {
        assert_eq!(file_date_type_parser("c").unwrap(), FileDateType::Created);
        assert_eq!(file_date_type_parser("m").unwrap(), FileDateType::Modified);
        assert_eq!(file_date_type_parser("a").unwrap(), FileDateType::Accessed);
    }

    #[test]
    fn test_file_date_type_parser_case_insensitive() {
        assert_eq!(file_date_type_parser("CREATED").unwrap(), FileDateType::Created);
        assert_eq!(file_date_type_parser("Modified").unwrap(), FileDateType::Modified);
        assert_eq!(file_date_type_parser("ACCessed").unwrap(), FileDateType::Accessed);
        assert_eq!(file_date_type_parser("C").unwrap(), FileDateType::Created);
        assert_eq!(file_date_type_parser("M").unwrap(), FileDateType::Modified);
        assert_eq!(file_date_type_parser("A").unwrap(), FileDateType::Accessed);
    }

    #[test]
    fn test_file_date_type_parser_whitespace_handling() {
        assert_eq!(file_date_type_parser(" created ").unwrap(), FileDateType::Created);
        assert_eq!(file_date_type_parser("  m  ").unwrap(), FileDateType::Modified);
        assert_eq!(file_date_type_parser("\taccessed\t").unwrap(), FileDateType::Accessed);
    }

    #[test]
    fn test_file_date_type_parser_invalid_inputs() {
        assert!(file_date_type_parser("invalid").is_err());
        assert!(file_date_type_parser("").is_err());
        assert!(file_date_type_parser("cm").is_err());
        assert!(file_date_type_parser("x").is_err());
        assert!(file_date_type_parser("create").is_err()); // typo
        assert!(file_date_type_parser("modify").is_err()); // wrong word
    }

    #[test]
    fn test_file_date_type_parser_error_message() {
        let result = file_date_type_parser("invalid");
        assert!(result.is_err());
        let error = result.unwrap_err();
        assert!(error.contains("Unsupported file date type"));
        assert!(error.contains("created (c)"));
        assert!(error.contains("modified (m)"));
        assert!(error.contains("accessed (a)"));
    }
}

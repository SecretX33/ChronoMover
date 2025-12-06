use crate::model::{Args, CollisionStrategy, FileToMove, GroupBy};
use crate::{date, log};
use chrono::{DateTime, Utc};
use color_eyre::eyre::{bail, Context, Result};
use date::{get_biweekly_identifier, get_file_date, get_month_identifier, get_quadrimester_identifier, get_semester_identifier, get_trimester_identifier, get_week_identifier, get_year_identifier};
use std::fs;
use std::path::{Path, PathBuf};
use walkdir::{DirEntry, WalkDir};

#[derive(Debug, Default)]
struct MoveStats {
    success: usize,
    skipped: usize,
    renamed: usize,
    failed: usize,
    collisions: usize,
}

pub fn get_files_to_move(args: &Args, now: DateTime<Utc>) -> Vec<FileToMove> {
    let mut files_to_move: Vec<FileToMove> = Vec::new();

    log!("Finding files to move in target folder...");

    let files_inside_target_folder = walk_source_folder(args)
        .filter_map(|entry| {
            if let Err(error) = &entry {
                log!("Failed to read entry: {:?}", error);
            }
            entry.ok()
        })
        .filter(|e| e.file_type().is_file())
        .filter(|e| {
            // Skip files in ignored paths
            let is_inside_ignored_folder = args.ignored_paths.as_ref()
                .is_some_and(|ignored_paths| is_inside_ignored_path(e.path(), ignored_paths));
            !is_inside_ignored_folder
        });
    
    for entry in files_inside_target_folder {
        let path = entry.path();

        // Get file date
        match get_file_date(path, &args.file_date_types) {
            Ok(file_datetime) => {
                // Determine if file should be moved
                if should_move_file(
                    file_datetime,
                    args.group_by,
                    args.previous_period_only,
                    args.older_than,
                    now,
                ) {
                    // Get the group identifier if grouping is enabled
                    let group_folder = match args.group_by {
                        Some(GroupBy::Week) => Some(get_week_identifier(file_datetime)),
                        Some(GroupBy::Month) => Some(get_month_identifier(file_datetime)),
                        Some(GroupBy::Year) => Some(get_year_identifier(file_datetime)),
                        Some(GroupBy::Semester) => Some(get_semester_identifier(file_datetime)),
                        Some(GroupBy::Trimester) => Some(get_trimester_identifier(file_datetime)),
                        Some(GroupBy::Quadrimester) => Some(get_quadrimester_identifier(file_datetime)),
                        Some(GroupBy::Biweekly) => Some(get_biweekly_identifier(file_datetime)),
                        None => None,
                    };

                    // Calculate destination path
                    match calculate_dest_path(
                        path,
                        &args.source,
                        &args.destination,
                        group_folder.as_deref()
                    ) {
                        Ok(dest_path) => {
                            log!("{}. {}",
                                files_to_move.len() + 1,
                                path.display()
                            );

                            let file_to_move = FileToMove {
                                source: path.to_path_buf(),
                                destination: dest_path,
                            };
                            files_to_move.push(file_to_move);
                        }
                        Err(e) => {
                            log!("WARNING: Failed to calculate destination for {}: {}", path.display(), e);
                        }
                    }
                }
            }
            Err(e) => {
                log!("WARNING: Failed to get file date for {}: {}", path.display(), e);
            }
        }
    }

    log!("Found {} file(s) to move", files_to_move.len());

    files_to_move
}

fn walk_source_folder(args: &Args) -> impl Iterator<Item = Result<DirEntry>> {
    let mut walk = WalkDir::new(&args.source).follow_links(args.follow_symbolic_links);

    if let Some(min_depth) = args.min_depth {
        walk = walk.min_depth(min_depth);
    }
    if let Some(max_depth) = args.max_depth {
        walk = walk.max_depth(max_depth);
    }

    walk.into_iter()
        .map(|e| e.map_err(Into::into))
}

/// Check if a path is inside any of the given parent paths.
/// Uses canonicalization and case-insensitive comparison on Windows/macOS.
fn is_inside_ignored_path(path: &Path, ignored_paths: &[PathBuf]) -> bool {
    // Canonicalize the path being checked (fallback to original if canonicalization fails)
    let canonical_path = dunce::canonicalize(path).unwrap_or_else(|_| path.to_path_buf());

    ignored_paths.iter()
        .map(|ignored_path| dunce::canonicalize(ignored_path).unwrap_or_else(|_| ignored_path.clone()))
        .any(|canonical_ignored_path| {
            if cfg!(any(target_os = "windows", target_os = "macos")) {
                // Case-insensitive comparison on Windows/macOS
                let path_str = canonical_path.to_string_lossy().to_lowercase();
                let ignored_str = canonical_ignored_path.to_string_lossy().to_lowercase();
                path_str.starts_with(&ignored_str)
            } else {
                canonical_path.starts_with(&canonical_ignored_path)
            }
        })
}

/// Determine if a file should be moved based on filters
fn should_move_file(
    file_datetime: DateTime<Utc>,
    group_by: Option<GroupBy>,
    previous_period_only: bool,
    older_than: Option<DateTime<Utc>>,
    now: DateTime<Utc>,
) -> bool {
    // Check older_than filter if specified
    if let Some(cutoff) = older_than
        && file_datetime >= cutoff {
            return false;
        }

    // Check previous_period_only filter if specified
    if previous_period_only {
        if let Some(group) = group_by {
            let is_before_current = match group {
                GroupBy::Week => date::is_before_current_week(file_datetime, now),
                GroupBy::Month => date::is_before_current_month(file_datetime, now),
                GroupBy::Year => date::is_before_current_year(file_datetime, now),
                GroupBy::Semester => date::is_before_current_semester(file_datetime, now),
                GroupBy::Trimester => date::is_before_current_trimester(file_datetime, now),
                GroupBy::Quadrimester => date::is_before_current_quadrimester(file_datetime, now),
                GroupBy::Biweekly => date::is_before_current_biweekly(file_datetime, now),
            };
            if !is_before_current {
                return false;
            }
        } else {
            // previous_period_only without group_by doesn't make sense, but we'll allow it
            // and just ignore the flag
        }
    }

    // If no filters apply, move the file
    true
}

/// Calculate destination path for a file
fn calculate_dest_path(
    source_path: &Path,
    source_root: &Path,
    dest_root: &Path,
    group_folder: Option<&str>,
) -> Result<PathBuf> {
    // Get the relative path from the source root
    let relative_path = source_path
        .strip_prefix(source_root)
        .context("Failed to compute relative path")?;

    // Construct the destination path
    let dest_path = if let Some(group) = group_folder {
        // Add grouping folder between destination root and relative path
        dest_root.join(group).join(relative_path)
    } else {
        // No grouping, just append relative path
        dest_root.join(relative_path)
    };

    Ok(dest_path)
}

/// Validate that no destination files already exist (for Fail strategy)
pub fn validate_no_collisions(files_to_move: &[FileToMove]) -> Result<()> {
    let collisions: Vec<&FileToMove> = files_to_move
        .iter()
        .filter(|f| f.destination.exists())
        .collect();

    if !collisions.is_empty() {
        let collision_list = collisions.iter()
            .take(5)
            .map(|f| format!("  - {} -> {}", f.source.display(), f.destination.display()))
            .collect::<Vec<_>>()
            .join("\n");

        let more = if collisions.len() > 5 {
            format!("\n  ... and {} more", collisions.len() - 5)
        } else {
            String::new()
        };

        bail!(
            "Cannot proceed: {} file(s) already exist at destination.\n\n\
            Collisions found:\n{}{}\n\n\
            Options to resolve:\n\
            1. Use --collision-strategy skip      - Skip conflicting files, move the rest\n\
            2. Use --collision-strategy overwrite - Replace existing files at destination\n\
            3. Use --collision-strategy rename    - Add numeric suffix to moved files (file.txt -> file_1.txt)\n\
            4. Manually remove conflicting files from destination\n\
            5. Use --dry-run to preview which files would collide",
            collisions.len(),
            collision_list,
            more
        );
    }
    Ok(())
}

/// Find a unique path by adding numeric suffix (file.txt -> file_1.txt, file_2.txt, etc.)
fn find_unique_path(path: &Path) -> PathBuf {
    let stem = path.file_stem().and_then(|s| s.to_str()).unwrap_or("");
    let ext = path.extension().and_then(|s| s.to_str());
    let parent = path.parent().unwrap_or(Path::new(""));

    let mut counter = 1;
    loop {
        let new_name = match ext {
            Some(e) => format!("{}_{}.{}", stem, counter, e),
            None => format!("{}_{}", stem, counter),
        };
        let new_path = parent.join(new_name);
        if !new_path.exists() {
            return new_path;
        }
        counter += 1;
    }
}

/// Execute the move plan (or preview in dry-run mode)
pub fn move_files(
    args: &Args,
    files_to_move: &[FileToMove],
    dry_run: bool,
) -> Result<()> {
    // Pre-validate for Fail strategy
    if !dry_run && args.collision_strategy == CollisionStrategy::Fail {
        validate_no_collisions(files_to_move)?;
    }

    if !files_to_move.is_empty() {
        log!("\nMoving files{}...", if dry_run { " (DRY RUN)" } else { "" } );
    }

    let mut move_stats = MoveStats::default();
    let max = files_to_move.len();

    for (index, item) in files_to_move.iter().enumerate() {
        let source_path = &item.source;
        let mut destination_path = item.destination.clone();

        // Check for collision
        let file_already_exists = destination_path.exists();

        if file_already_exists {
            move_stats.collisions += 1;
        }

        if !dry_run {
            // Handle collision based on strategy
            if file_already_exists {
                match args.collision_strategy {
                    CollisionStrategy::Skip => {
                        log!("SKIP: {} (destination exists)", source_path.display());
                        move_stats.skipped += 1;
                        continue;
                    }
                    CollisionStrategy::Overwrite => {
                        fs::remove_file(&destination_path)
                            .with_context(|| format!("Failed to remove existing file: {}", destination_path.display()))?;
                    }
                    CollisionStrategy::Rename => {
                        let original_dest = destination_path.clone();
                        destination_path = find_unique_path(&destination_path);
                        log!("WARNING: Renamed {} -> {} (destination existed at {})",
                            source_path.display(),
                            destination_path.file_name().unwrap_or_default().to_string_lossy(),
                            original_dest.display());
                        move_stats.renamed += 1;
                    }
                    CollisionStrategy::Fail => {
                        // Already validated above, but handle edge case
                        bail!("Destination exists: {}", destination_path.display());
                    }
                }
            }

            // Create parent directories if they don't exist
            if let Some(parent) = destination_path.parent() {
                fs::create_dir_all(parent)
                    .with_context(|| format!("Failed to create directory: {}", parent.display()))?;
            }

            // Move the file
            if let Err(e) = fs::rename(source_path, &destination_path) {
                log!("ERROR: Moving file {}: {}", source_path.display(), e);
                move_stats.failed += 1;
                continue;
            }
            move_stats.success += 1;
        }

        if !file_already_exists {
            log!(
                "{}/{}. {}\n       ↳ {}",
                index + 1,
                max,
                source_path.display(),
                destination_path.parent().map(|it| it.display()).unwrap_or(destination_path.display())
            );
        } else if dry_run {
            log!(
                "{}/{}. COLLISION: {}\n       ↳ {} (destination exists)",
                index + 1,
                max,
                source_path.display(),
                destination_path.display()
            );
        }
    }

    if args.dry_run {
        log!("DRY RUN: {} file(s) would have been moved successfully", move_stats.success);
    } else {
        log!("Finished moving files, {} file(s) moved successfully", move_stats.success);
    }

    // Report failed/skipped/renamed files prominently
    if move_stats.failed > 0 {
        log!("\nWARNING: {} file(s) failed to move", move_stats.failed);
    }
    if move_stats.skipped > 0 {
        log!("INFO: {} file(s) skipped (destination exists)", move_stats.skipped);
    }
    if move_stats.renamed > 0 {
        log!("INFO: {} file(s) renamed to avoid collision", move_stats.renamed);
    }
    if dry_run && move_stats.collisions > 0 {
        log!("WARNING: {} file(s) would collide with existing files at destination", move_stats.collisions);
    }

    Ok(())
}

/// Delete empty directories recursively
pub fn delete_empty_directories(args: &Args, root: &Path) -> Result<()> {
    if args.dry_run || args.keep_empty_folders {
        return Ok(());
    }

    let mut deleted_dirs = Vec::new();

    // We need to process directories from deepest to shallowest
    // to properly handle nested empty directories
    loop {
        let mut found_empty = false;

        for entry in WalkDir::new(root)
            .min_depth(1)
            .follow_links(args.follow_symbolic_links)
            .into_iter()
            .filter_map(Result::ok)
            .filter(|e| e.file_type().is_dir())
        {
            let path = entry.path();

            // Skip ignored paths
            let is_inside_ignored_folder = args.ignored_paths.as_ref()
                .is_some_and(|ignored_paths| is_inside_ignored_path(path, ignored_paths));
            if is_inside_ignored_folder {
                continue;
            }

            // Check if directory is empty
            if let Ok(mut entries) = fs::read_dir(path)
                && entries.next().is_none() {
                    // Directory is empty, delete it
                    fs::remove_dir(path)
                        .with_context(|| format!("Failed to delete empty directory: {}", path.display()))?;
                    deleted_dirs.push(path.to_path_buf());
                    found_empty = true;
                }
        }

        // If we didn't find any empty directories, we're done
        if !found_empty {
            break;
        }
    }

    if !deleted_dirs.is_empty() {
        log!("\nCleaning up empty directories...");
        for (index, dir) in deleted_dirs.iter().enumerate() {
            log!("{}/{}. Deleted empty directory: {}", index + 1, deleted_dirs.len(), dir.display());
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    // should_move_file tests
    #[test]
    fn test_should_move_file_no_filters() {
        let now = "2025-06-15T00:00:00Z".parse::<DateTime<Utc>>().unwrap();
        let file_datetime = "2025-01-01T12:00:00Z".parse::<DateTime<Utc>>().unwrap();

        // With no filters, should always move
        assert!(should_move_file(file_datetime, None, false, None, now));
    }

    #[test]
    fn test_should_move_file_older_than_filter() {
        let now = "2025-06-15T00:00:00Z".parse::<DateTime<Utc>>().unwrap();
        let cutoff = "2025-03-01T00:00:00Z".parse::<DateTime<Utc>>().unwrap();

        // File before cutoff - should move
        let before_cutoff = "2025-02-15T12:00:00Z".parse::<DateTime<Utc>>().unwrap();
        assert!(should_move_file(before_cutoff, None, false, Some(cutoff), now));

        // File after cutoff - should not move
        let after_cutoff = "2025-03-15T12:00:00Z".parse::<DateTime<Utc>>().unwrap();
        assert!(!should_move_file(after_cutoff, None, false, Some(cutoff), now));

        // File exactly at cutoff - should not move (>= comparison)
        let at_cutoff = "2025-03-01T00:00:00Z".parse::<DateTime<Utc>>().unwrap();
        assert!(!should_move_file(at_cutoff, None, false, Some(cutoff), now));
    }

    #[test]
    fn test_should_move_file_previous_period_only_week() {
        let now = "2025-06-15T00:00:00Z".parse::<DateTime<Utc>>().unwrap(); // Week 24

        // Previous week - should move
        let previous_week = "2025-06-08T12:00:00Z".parse::<DateTime<Utc>>().unwrap();
        assert!(should_move_file(previous_week, Some(GroupBy::Week), true, None, now));

        // Current week - should not move
        let current_week = "2025-06-16T12:00:00Z".parse::<DateTime<Utc>>().unwrap();
        assert!(!should_move_file(current_week, Some(GroupBy::Week), true, None, now));

        // Next week - should not move
        let next_week = "2025-06-22T12:00:00Z".parse::<DateTime<Utc>>().unwrap();
        assert!(!should_move_file(next_week, Some(GroupBy::Week), true, None, now));
    }

    #[test]
    fn test_should_move_file_previous_period_only_month() {
        let now = "2025-06-15T00:00:00Z".parse::<DateTime<Utc>>().unwrap(); // June

        // Previous month - should move
        let previous_month = "2025-05-31T12:00:00Z".parse::<DateTime<Utc>>().unwrap();
        assert!(should_move_file(previous_month, Some(GroupBy::Month), true, None, now));

        // Current month - should not move
        let current_month = "2025-06-01T12:00:00Z".parse::<DateTime<Utc>>().unwrap();
        assert!(!should_move_file(current_month, Some(GroupBy::Month), true, None, now));

        // Next month - should not move
        let next_month = "2025-07-01T12:00:00Z".parse::<DateTime<Utc>>().unwrap();
        assert!(!should_move_file(next_month, Some(GroupBy::Month), true, None, now));
    }

    #[test]
    fn test_should_move_file_previous_period_only_year() {
        let now = "2025-06-15T00:00:00Z".parse::<DateTime<Utc>>().unwrap(); // 2025

        // Previous year - should move
        let previous_year = "2024-12-31T12:00:00Z".parse::<DateTime<Utc>>().unwrap();
        assert!(should_move_file(previous_year, Some(GroupBy::Year), true, None, now));

        // Current year - should not move
        let current_year = "2025-01-01T12:00:00Z".parse::<DateTime<Utc>>().unwrap();
        assert!(!should_move_file(current_year, Some(GroupBy::Year), true, None, now));

        // Next year - should not move
        let next_year = "2026-01-01T12:00:00Z".parse::<DateTime<Utc>>().unwrap();
        assert!(!should_move_file(next_year, Some(GroupBy::Year), true, None, now));
    }

    #[test]
    fn test_should_move_file_previous_period_only_semester() {
        let now = "2025-08-15T00:00:00Z".parse::<DateTime<Utc>>().unwrap(); // H2

        // Previous semester (H1) - should move
        let previous_semester = "2025-06-30T12:00:00Z".parse::<DateTime<Utc>>().unwrap();
        assert!(should_move_file(previous_semester, Some(GroupBy::Semester), true, None, now));

        // Current semester (H2) - should not move
        let current_semester = "2025-08-01T12:00:00Z".parse::<DateTime<Utc>>().unwrap();
        assert!(!should_move_file(current_semester, Some(GroupBy::Semester), true, None, now));
    }

    #[test]
    fn test_should_move_file_previous_period_only_trimester() {
        let now = "2025-05-15T00:00:00Z".parse::<DateTime<Utc>>().unwrap(); // Q2

        // Previous trimester (Q1) - should move
        let previous_trimester = "2025-03-31T12:00:00Z".parse::<DateTime<Utc>>().unwrap();
        assert!(should_move_file(previous_trimester, Some(GroupBy::Trimester), true, None, now));

        // Current trimester (Q2) - should not move
        let current_trimester = "2025-05-01T12:00:00Z".parse::<DateTime<Utc>>().unwrap();
        assert!(!should_move_file(current_trimester, Some(GroupBy::Trimester), true, None, now));
    }

    #[test]
    fn test_should_move_file_previous_period_only_quadrimester() {
        let now = "2025-06-15T00:00:00Z".parse::<DateTime<Utc>>().unwrap(); // QD2

        // Previous quadrimester (QD1) - should move
        let previous_qd = "2025-04-30T12:00:00Z".parse::<DateTime<Utc>>().unwrap();
        assert!(should_move_file(previous_qd, Some(GroupBy::Quadrimester), true, None, now));

        // Current quadrimester (QD2) - should not move
        let current_qd = "2025-05-01T12:00:00Z".parse::<DateTime<Utc>>().unwrap();
        assert!(!should_move_file(current_qd, Some(GroupBy::Quadrimester), true, None, now));
    }

    #[test]
    fn test_should_move_file_previous_period_only_biweekly() {
        let now = "2025-06-15T00:00:00Z".parse::<DateTime<Utc>>().unwrap(); // Week 24 -> BW12

        // Previous biweekly period - should move
        let previous_bw = "2025-06-01T12:00:00Z".parse::<DateTime<Utc>>().unwrap();
        assert!(should_move_file(previous_bw, Some(GroupBy::Biweekly), true, None, now));

        // Current biweekly period - should not move
        let current_bw = "2025-06-16T12:00:00Z".parse::<DateTime<Utc>>().unwrap();
        assert!(!should_move_file(current_bw, Some(GroupBy::Biweekly), true, None, now));
    }

    #[test]
    fn test_should_move_file_combined_filters() {
        let now = "2025-06-15T00:00:00Z".parse::<DateTime<Utc>>().unwrap(); // Week 24
        let cutoff = "2025-06-10T00:00:00Z".parse::<DateTime<Utc>>().unwrap();

        // Passes both filters: before cutoff (June 8) AND previous period (Week 23)
        let passes_both = "2025-06-08T12:00:00Z".parse::<DateTime<Utc>>().unwrap();
        assert!(should_move_file(passes_both, Some(GroupBy::Week), true, Some(cutoff), now));

        // Fails older_than: after cutoff (June 14) but in previous period (Week 23)
        // Note: June 14 is actually in Week 24, so let me use Week 23 date after cutoff
        // Week 23 is June 2-8, so June 14 is Week 24. Let me use a date from previous week that's after cutoff
        // Actually, if cutoff is June 10, then all dates in Week 23 (June 2-8) are before cutoff
        // Let's use Month grouping instead for this test case
        let now_month = "2025-06-15T00:00:00Z".parse::<DateTime<Utc>>().unwrap(); // June
        let cutoff_month = "2025-05-15T00:00:00Z".parse::<DateTime<Utc>>().unwrap();

        // Fails older_than: after cutoff (May 20) but in previous period (May)
        let fails_older_than = "2025-05-20T12:00:00Z".parse::<DateTime<Utc>>().unwrap();
        assert!(!should_move_file(fails_older_than, Some(GroupBy::Month), true, Some(cutoff_month), now_month));

        // Fails previous_period_only: before cutoff (June 5) but in current period (June)
        let fails_period = "2025-06-05T12:00:00Z".parse::<DateTime<Utc>>().unwrap();
        assert!(!should_move_file(fails_period, Some(GroupBy::Month), true, Some(cutoff_month), now_month));

        // Fails both filters: after cutoff AND in current period
        let fails_both = "2025-06-16T12:00:00Z".parse::<DateTime<Utc>>().unwrap();
        assert!(!should_move_file(fails_both, Some(GroupBy::Month), true, Some(cutoff_month), now_month));
    }

    #[test]
    fn test_should_move_file_previous_period_only_without_group_by() {
        let now = "2025-06-15T00:00:00Z".parse::<DateTime<Utc>>().unwrap();
        let file_datetime = "2025-01-01T12:00:00Z".parse::<DateTime<Utc>>().unwrap();

        // previous_period_only without group_by should be ignored, file should move
        assert!(should_move_file(file_datetime, None, true, None, now));
    }

    // calculate_dest_path tests
    #[test]
    fn test_calculate_dest_path_without_grouping() {
        // Root-level file
        let source_root = PathBuf::from("/source");
        let dest_root = PathBuf::from("/dest");
        let source_path = source_root.join("file.md");

        let result = calculate_dest_path(&source_path, &source_root, &dest_root, None).unwrap();
        assert_eq!(result, dest_root.join("file.md"));

        // Nested file
        let nested_source = source_root.join("folder1").join("folder2").join("file.md");
        let result = calculate_dest_path(&nested_source, &source_root, &dest_root, None).unwrap();
        assert_eq!(result, dest_root.join("folder1").join("folder2").join("file.md"));
    }

    #[test]
    fn test_calculate_dest_path_with_grouping() {
        let source_root = PathBuf::from("/source");
        let dest_root = PathBuf::from("/dest");
        let group_folder = "2025-24";

        // Root-level file
        let source_path = source_root.join("file.md");
        let result = calculate_dest_path(&source_path, &source_root, &dest_root, Some(group_folder)).unwrap();
        assert_eq!(result, dest_root.join(group_folder).join("file.md"));

        // Nested file
        let nested_source = source_root.join("folder1").join("folder2").join("file.md");
        let result = calculate_dest_path(&nested_source, &source_root, &dest_root, Some(group_folder)).unwrap();
        assert_eq!(result, dest_root.join(group_folder).join("folder1").join("folder2").join("file.md"));
    }

    #[test]
    fn test_calculate_dest_path_preserves_structure() {
        let source_root = PathBuf::from("/notes");
        let dest_root = PathBuf::from("/archive");

        // Test with various nesting levels
        let paths = vec![
            "daily.md",
            "work/meeting.md",
            "work/projects/project1.md",
            "personal/journal/2025/january.md",
        ];

        for path in paths {
            let source_path = source_root.join(path);
            let result = calculate_dest_path(&source_path, &source_root, &dest_root, None).unwrap();
            assert_eq!(result, dest_root.join(path));
        }
    }

    #[test]
    fn test_calculate_dest_path_with_grouping_preserves_structure() {
        let source_root = PathBuf::from("/notes");
        let dest_root = PathBuf::from("/archive");
        let group = "2025-W24";

        let paths = vec![
            "daily.md",
            "work/meeting.md",
            "work/projects/project1.md",
        ];

        for path in paths {
            let source_path = source_root.join(path);
            let result = calculate_dest_path(&source_path, &source_root, &dest_root, Some(group)).unwrap();
            assert_eq!(result, dest_root.join(group).join(path));
        }
    }

    #[test]
    fn test_calculate_dest_path_different_group_formats() {
        let source_root = PathBuf::from("/source");
        let dest_root = PathBuf::from("/dest");
        let source_path = source_root.join("file.md");

        // Test with different grouping formats
        let groups = vec![
            "2025-24",      // Week
            "2025-06",      // Month
            "2025",         // Year
            "2025-H1",      // Semester
            "2025-Q2",      // Trimester
            "2025-QD2",     // Quadrimester
            "2025-BW12",    // Biweekly
        ];

        for group in groups {
            let result = calculate_dest_path(&source_path, &source_root, &dest_root, Some(group)).unwrap();
            assert_eq!(result, dest_root.join(group).join("file.md"));
        }
    }
}

# ChronoMover - Advanced Documentation

This document contains technical details, build instructions, scheduling guides, and advanced usage examples for ChronoMover.

## Table of Contents

- [Building from Source](#building-from-source)
- [Grouping Strategies](#grouping-strategies)
- [Advanced Filtering](#advanced-filtering)
- [File Timestamp Types](#file-timestamp-types)
- [Empty Folder Cleanup](#empty-folder-cleanup)
- [Path Filtering and Traversal Control](#path-filtering-and-traversal-control)
- [Advanced Usage Examples](#advanced-usage-examples)
- [Scheduling Automatic Runs](#scheduling-automatic-runs)
- [Troubleshooting](#troubleshooting)
- [Development Commands](#development-commands)

## Building from Source

If you want to build ChronoMover yourself or contribute to development:

### Requirements

- **Rust** (stable toolchain)
- **Cargo** (comes with Rust)
- Compatible with Windows, macOS, and Linux

### Installation Steps

1. **Install Rust** (if not already installed):
   - Download from: https://rustup.rs/
   - Run the installer and follow the prompts
   - Restart your terminal after installation

2. **Clone or download the repository**:
   ```bash
   git clone https://github.com/SecretX33/ChronoMover.git
   cd ChronoMover
   ```

3. **Build the application**:
   ```bash
   cargo build --release
   ```

4. **Find the compiled executable**:
   - **Windows**: `target\release\chronomover.exe`
   - **macOS/Linux**: `target/release/chronomover`

### Running from Source

You can run the application directly with `cargo run`:

```bash
# With cargo
cargo run --release -- --source "C:\Notes" --destination "C:\Archive" --dry-run

# Or run the compiled executable directly
# Windows:
.\target\release\chronomover.exe --source "C:\Notes" --destination "C:\Archive"

# macOS/Linux:
./target/release/chronomover --source "$HOME/Notes" --destination "$HOME/Archive"
```

## Grouping Strategies

ChronoMover supports 7 different grouping strategies for organizing files into time-based folders.

### Week (ISO 8601)

Groups files by ISO week number. Weeks start on Monday, and Week 1 is the first week with a Thursday in the new year.

**Format**: `YYYY-WNN` (e.g., `2025-W01`, `2025-W52`)

```bash
chronomover --source "C:\Notes" --destination "C:\Archive" --group-by week
```

**Example folder structure:**
```
Archive/
├── 2025-W44/
│   ├── Meeting.md
│   └── Notes.md
├── 2025-W45/
└── 2025-W46/
```

### Biweekly

Groups files into 26 two-week periods per year.

**Format**: `YYYY-BWNN` (e.g., `2025-BW01` through `2025-BW26`)

```bash
chronomover --source "C:\Notes" --destination "C:\Archive" --group-by biweekly
```

### Month

Groups files by calendar month.

**Format**: `YYYY-MM` (e.g., `2025-01`, `2025-12`)

```bash
chronomover --source "C:\Notes" --destination "C:\Archive" --group-by month
```

**Example folder structure:**
```
Archive/
├── 2025-10/
├── 2025-11/
└── 2025-12/
```

### Trimester (Quarter)

Groups files into 4 quarters (3 months each).

**Format**: `YYYY-QN` (e.g., `2025-Q1` through `2025-Q4`)

- Q1: January-March
- Q2: April-June
- Q3: July-September
- Q4: October-December

```bash
chronomover --source "C:\Notes" --destination "C:\Archive" --group-by trimester
```

### Quadrimester

Groups files into 3 quadrimesters (4 months each).

**Format**: `YYYY-QDN` (e.g., `2025-QD1` through `2025-QD3`)

- QD1: January-April
- QD2: May-August
- QD3: September-December

```bash
chronomover --source "C:\Notes" --destination "C:\Archive" --group-by quadrimester
```

### Semester

Groups files into 2 semesters (6 months each).

**Format**: `YYYY-HN` (e.g., `2025-H1`, `2025-H2`)

- H1: January-June
- H2: July-December

```bash
chronomover --source "C:\Notes" --destination "C:\Archive" --group-by semester
```

### Year

Groups files by calendar year.

**Format**: `YYYY` (e.g., `2024`, `2025`)

```bash
chronomover --source "C:\Notes" --destination "C:\Archive" --group-by year
```

## Advanced Filtering

ChronoMover provides flexible filtering options to control which files get moved.

### Previous Period Only

When using `--group-by`, the `--previous-period-only` flag excludes files from the current period.

**Examples:**

```bash
# Move only files from previous weeks (not the current week)
chronomover --source "C:\Notes" --destination "C:\Archive" --group-by week --previous-period-only

# Move only files from previous months (not the current month)
chronomover --source "C:\Notes" --destination "C:\Archive" --group-by month --previous-period-only

# Move only files from previous years (not the current year)
chronomover --source "C:\Notes" --destination "C:\Archive" --group-by year --previous-period-only
```

**Note**: This flag requires `--group-by`. Using it without grouping will show a warning.

### Older Than Filter

The `--older-than` filter moves only files whose timestamps are older than the specified cutoff.

#### Duration Format

Human-readable duration formats are supported:

| Unit     | Examples                 | Description                  |
|----------|--------------------------|------------------------------|
| Days     | `30d`, `7d`              | Number of days               |
| Weeks    | `2w`, `4w`               | Number of weeks              |
| Months   | `6M`, `3M`               | Number of months (capital M) |
| Years    | `1y`, `2y`               | Number of years              |
| Combined | `1y6M`, `2w3d`, `1y2M3d` | Multiple units combined      |

**Examples:**

```bash
# Files older than 30 days
chronomover --source "C:\Notes" --destination "C:\Archive" --older-than 30d

# Files older than 1 year
chronomover --source "C:\Notes" --destination "C:\Archive" --older-than 1y

# Files older than 1 year and 6 months
chronomover --source "C:\Notes" --destination "C:\Archive" --older-than 1y6M
```

#### ISO Date Format

You can also specify exact dates:

```bash
# Files older than January 1, 2025
chronomover --source "C:\Notes" --destination "C:\Archive" --older-than 2025-01-01

# Files older than October 15, 2024
chronomover --source "C:\Notes" --destination "C:\Archive" --older-than 2024-10-15
```

#### ISO DateTime Format

For precise cutoffs with time:

```bash
# Files older than a specific date and time
chronomover --source "C:\Notes" --destination "C:\Archive" --older-than "2025-01-01T09:00:00"
```

#### How Cutoff Times Are Calculated

The cutoff time is determined differently based on the format:

**Duration** - Current time minus the duration equals the cutoff:
- If now is `2025-11-05 10:00:00` and `--older-than 30d`
- Cutoff is `2025-10-06 10:00:00`
- Files with timestamps before this are moved

**ISO Date** - Midnight at the start of the specified date:
- `--older-than 2025-01-15` = cutoff at `2025-01-15 00:00:00`
- Files dated before January 15 are moved

**ISO DateTime** - Exact specified time in local timezone:
- `--older-than 2025-01-15T14:30:00` = cutoff at `2025-01-15 14:30:00`
- Files with timestamps before 2:30 PM are moved

### Combining Filters

You can combine `--previous-period-only` and `--older-than` for precise control. Both conditions must be met (AND logic).

```bash
# Files from previous weeks AND older than 90 days
chronomover --source "C:\Notes" --destination "C:\Archive" --group-by week --previous-period-only --older-than 90d

# Files from previous months AND older than 6 months
chronomover --source "C:\Notes" --destination "C:\Archive" --group-by month --previous-period-only --older-than 6M
```

## File Timestamp Types

ChronoMover can use different file timestamps to determine when a file should be moved. Use the `--file-date-types` option to control which timestamps to check.

### Available Timestamp Types

- **`created`** (or `c`): File creation time
- **`modified`** (or `m`): File last modification time
- **`accessed`** (or `a`): File last access time

### Default Behavior

By default, ChronoMover checks both `created` and `modified` timestamps and uses the **most recent** one. This prevents accidentally archiving files that were created long ago but recently modified.

### Custom Timestamp Selection

**Use only modification date:**
```bash
chronomover --source "C:\Notes" --destination "C:\Archive" --file-date-types modified
# or shorthand:
chronomover --source "C:\Notes" --destination "C:\Archive" --file-date-types m
```

**Use all three timestamps (picks most recent):**
```bash
chronomover --source "C:\Notes" --destination "C:\Archive" --file-date-types c,m,a
```

**Use creation date only:**
```bash
chronomover --source "C:\Notes" --destination "C:\Archive" --file-date-types created
```

### Platform Considerations

- **Windows**: All three timestamp types are fully supported
- **macOS/Linux**: File creation time may not be reliably available on all filesystems. The app falls back to modification time when creation time is unavailable.

## Path Filtering and Traversal Control

ChronoMover provides fine-grained control over which files are processed through path filtering and directory traversal options.

### Ignored Paths

Use `--ignored-paths` to exclude specific files or directories from being moved. This is useful for protecting important folders or files that should never be archived.

**Format**: Comma-separated list of absolute paths

**Examples:**

```bash
# Ignore a single directory
chronomover --source "C:\Notes" --destination "C:\Archive" --ignored-paths "C:\Notes\Important"

# Ignore multiple directories
chronomover --source "C:\Notes" --destination "C:\Archive" --ignored-paths "C:\Notes\Important,C:\Notes\Current,C:\Notes\InProgress"

# Ignore a specific file
chronomover --source "C:\Notes" --destination "C:\Archive" --ignored-paths "C:\Notes\README.md"
```

**Behavior:**
- Paths must be absolute (full paths from the root)
- Any file inside an ignored directory will also be skipped
- If an ignored path doesn't exist, a warning is logged but execution continues
- Ignored paths are checked before any filtering logic runs

**Practical use cases:**
- Exclude "Current" or "Active" project folders
- Protect configuration files or README files
- Skip temporary or cache directories
- Preserve specific important subdirectories

### Directory Depth Control

Control how deep ChronoMover searches for files using `--min-depth` and `--max-depth` options.

**Depth levels:**
- Depth 0: Files in the source directory root
- Depth 1: Files in immediate subdirectories
- Depth 2: Files in subdirectories of subdirectories
- And so on...

#### Minimum Depth

Only process files at or below the specified depth level.

```bash
# Only process files in subdirectories (skip root-level files)
chronomover --source "C:\Notes" --destination "C:\Archive" --min-depth 1

# Only process files in deeply nested folders (2+ levels deep)
chronomover --source "C:\Notes" --destination "C:\Archive" --min-depth 2
```

**Use cases:**
- Skip root-level files but process subdirectories
- Process only deeply nested files
- Exclude top-level files from archiving

#### Maximum Depth

Only process files up to the specified depth level.

```bash
# Only process root-level files (depth 0)
chronomover --source "C:\Notes" --destination "C:\Archive" --max-depth 0

# Process files up to 2 levels deep
chronomover --source "C:\Notes" --destination "C:\Archive" --max-depth 2
```

**Use cases:**
- Process only root-level files
- Avoid processing deeply nested subdirectories
- Limit scope for performance reasons
- Process flat directory structures

#### Combining Min and Max Depth

```bash
# Process only files at exactly depth 1 (immediate subdirectories only)
chronomover --source "C:\Notes" --destination "C:\Archive" --min-depth 1 --max-depth 1

# Process files at depths 2-3 only
chronomover --source "C:\Notes" --destination "C:\Archive" --min-depth 2 --max-depth 3
```

**Validation:**
- If both are specified, min-depth must be ≤ max-depth
- Invalid combinations will result in an error

### Symbolic Links

By default, ChronoMover does **not** follow symbolic links (symlinks). Use `--follow-symbolic-links` to change this behavior.

```bash
# Follow symbolic links during traversal
chronomover --source "C:\Notes" --destination "C:\Archive" --follow-symbolic-links
```

**Warning: Infinite Loops**

Following symbolic links can cause infinite loops if:
- A symlink points to a parent directory
- Symlinks create a cycle in the directory structure

**Best practices when using `--follow-symbolic-links`:**
- Ensure your directory structure doesn't have circular symlinks
- Use `--dry-run` first to verify behavior
- Consider using `--max-depth` to limit traversal
- Monitor the operation for unexpected behavior

**Platform considerations:**
- **Windows**: Symbolic links require administrator privileges to create, but can be followed by any user
- **macOS/Linux**: Symbolic links are common and well-supported
- Junction points (Windows) and hard links are treated differently by the filesystem

### Combining Traversal Options

You can combine all traversal options for precise control:

```bash
# Search depth 1-3, follow symlinks, ignore specific paths
chronomover \
  --source "C:\Projects" \
  --destination "C:\Archive" \
  --min-depth 1 \
  --max-depth 3 \
  --follow-symbolic-links \
  --ignored-paths "C:\Projects\Active,C:\Projects\Client-X" \
  --group-by month \
  --older-than 6M
```

This command:
- Skips root-level files (min-depth 1)
- Only searches up to 3 levels deep (max-depth 3)
- Follows symbolic links
- Ignores "Active" and "Client-X" directories
- Groups by month
- Only moves files older than 6 months

## Advanced Usage Examples

### Example 1: Weekly Archive with Previous Weeks Only

Archive notes from previous weeks, excluding the current week:

```bash
chronomover --source "C:\Notes" --destination "C:\Notes\Archive" --group-by week --previous-period-only --dry-run
```

Remove `--dry-run` when ready to move files.

### Example 2: Monthly Archive for Files Older Than 3 Months

Group by month, but only move files older than 3 months:

```bash
chronomover --source "C:\Documents" --destination "C:\Documents\Archive" --group-by month --older-than 3M
```

### Example 3: Yearly Archive for Long-Term Storage

Move files older than 2 years into yearly folders:

```bash
chronomover --source "C:\Work" --destination "D:\LongTermArchive" --group-by year --older-than 2y
```

### Example 4: Aggressive Archiving with Combined Filters

Archive by week, excluding current week, and only files older than 1 month:

```bash
chronomover --source "C:\Projects" --destination "C:\Projects\Archive" --group-by week --previous-period-only --older-than 1M
```

### Example 5: Semester-Based Academic Organization

Organize academic files by semester:

```bash
chronomover --source "C:\Courses" --destination "C:\Courses\Archive" --group-by semester --previous-period-only
```

### Example 6: Quarterly Reports Archiving

Organize quarterly reports with trimester grouping:

```bash
chronomover --source "C:\Reports" --destination "C:\Reports\Archive" --group-by trimester --older-than 6M
```

### Example 7: Using Only Modification Date

Archive files based solely on when they were last modified:

```bash
chronomover --source "C:\Notes" --destination "C:\Archive" --file-date-types modified --older-than 60d
```

### Example 8: Biweekly Archiving for Frequent Updates

For rapidly changing directories, use biweekly grouping:

```bash
chronomover --source "C:\Logs" --destination "C:\LogArchive" --group-by biweekly --previous-period-only
```

### Example 9: Archive Only Root-Level Files

Process only files in the root directory, ignoring all subdirectories:

```bash
chronomover --source "C:\Downloads" --destination "C:\Downloads\Archive" --max-depth 0 --older-than 30d
```

### Example 10: Protect Specific Folders While Archiving

Archive all old files except those in "Important" and "Current" folders:

```bash
chronomover --source "C:\Projects" --destination "C:\Archive" --older-than 3M --ignored-paths "C:\Projects\Important,C:\Projects\Current"
```

### Example 11: Process Only Immediate Subdirectories

Archive files only from first-level subdirectories, skipping root files and deeply nested files:

```bash
chronomover --source "C:\Documents" --destination "C:\Archive" --min-depth 1 --max-depth 1 --group-by month
```

### Example 12: Complex Multi-Filter Archiving

Combine multiple filters for precise control:

```bash
chronomover \
  --source "C:\WorkFiles" \
  --destination "D:\Archive" \
  --group-by month \
  --previous-period-only \
  --older-than 2M \
  --ignored-paths "C:\WorkFiles\ActiveProjects,C:\WorkFiles\Templates" \
  --min-depth 1 \
  --file-date-types modified \
  --dry-run
```

This command:
- Groups files by month
- Only processes previous months (not current month)
- Only moves files older than 2 months
- Ignores "ActiveProjects" and "Templates" folders
- Skips root-level files (min-depth 1)
- Uses only modification date
- Previews changes without moving

### Example 13: Preserve Empty Folders During Archiving

Archive files while keeping the original folder structure intact:

```bash
chronomover --source "C:\ProjectTemplates" --destination "C:\Archive" --older-than 1y --keep-empty-folders
```

This is useful when:
- Folder structure serves as a template for new projects
- Empty folders have organizational meaning
- Other processes rely on specific folders existing
- You want to manually clean up folders later

## Scheduling Automatic Runs

Set up ChronoMover to run automatically on a schedule. This is useful for maintaining a clean workspace without manual intervention.

### Windows (Task Scheduler)

#### Step 1: Prepare a Batch File

Create a file named `run-chronomover.bat` with your desired command:

```batch
@echo off
cd /d "C:\Tools\chronomover"
chronomover.exe --source "C:\Users\Me\Notes" --destination "C:\Users\Me\Notes\Archive" --group-by week --previous-period-only
```

Replace paths with your actual directories.

#### Step 2: Set Up Task Scheduler

1. **Open Task Scheduler:**
   - Press `Win + R`
   - Type `taskschd.msc` and press Enter

2. **Create a New Task:**
   - Click "Create Task" (not "Create Basic Task") in the right panel

3. **General Tab:**
   - Name: `ChronoMover`
   - Description: `Automatically archives files based on timestamps`
   - Select "Run whether user is logged on or not"
   - Check "Run with highest privileges"

4. **Triggers Tab:**
   - Click "New..."
   - Begin the task: `On a schedule`
   - Settings: `Weekly`
   - Select: `Monday` (or your preferred day)
   - Time: Choose your preferred time (e.g., `9:00 AM`)
   - Click "OK"

5. **Actions Tab:**
   - Click "New..."
   - Action: `Start a program`
   - Program/script: Browse to your `.bat` file or directly to `chronomover.exe`
   - Add arguments (if running exe directly): `--source "C:\Notes" --destination "C:\Archive" --group-by week`
   - Start in: The directory containing the executable (e.g., `C:\Tools\chronomover`)
   - Click "OK"

6. **Conditions Tab:**
   - Uncheck "Start the task only if the computer is on AC power" (if you want it to run on battery)

7. **Settings Tab:**
   - Check "Allow task to be run on demand"
   - Check "If the task fails, restart every: 1 minute" and set "Attempt to restart up to: 3 times"

8. **Click "OK"** and enter your Windows password if prompted

#### Step 3: Test the Task

1. Right-click on your task in Task Scheduler
2. Select "Run"
3. Check the "Last Run Result" column to verify it completed successfully (should show "The operation completed successfully (0x0)")

### macOS (launchd)

Create a launch agent at `~/Library/LaunchAgents/com.chronomover.plist`:

```xml
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>Label</key>
    <string>com.chronomover</string>
    <key>ProgramArguments</key>
    <array>
        <string>/Users/YOUR_USERNAME/tools/chronomover/chronomover</string>
        <string>--source</string>
        <string>/Users/YOUR_USERNAME/Notes</string>
        <string>--destination</string>
        <string>/Users/YOUR_USERNAME/Notes/Archive</string>
        <string>--group-by</string>
        <string>week</string>
        <string>--previous-period-only</string>
    </array>
    <key>StartCalendarInterval</key>
    <dict>
        <key>Weekday</key>
        <integer>1</integer>
        <key>Hour</key>
        <integer>9</integer>
        <key>Minute</key>
        <integer>0</integer>
    </dict>
</dict>
</plist>
```

**Replace `YOUR_USERNAME` with your actual username.**

Load the agent:
```bash
launchctl load ~/Library/LaunchAgents/com.chronomover.plist
```

To unload (stop automatic runs):
```bash
launchctl unload ~/Library/LaunchAgents/com.chronomover.plist
```

### Linux (cron)

Add a cron job by running `crontab -e` and adding:

```cron
# Run every Monday at 9:00 AM
0 9 * * 1 /home/YOUR_USERNAME/tools/chronomover/chronomover --source "/home/YOUR_USERNAME/Notes" --destination "/home/YOUR_USERNAME/Notes/Archive" --group-by week --previous-period-only
```

**Cron syntax**: `minute hour day month weekday command`

**Common schedules:**
```cron
# Every day at 9 AM
0 9 * * * /path/to/chronomover --source "..." --destination "..."

# Every Monday at 8:30 AM
30 8 * * 1 /path/to/chronomover --source "..." --destination "..."

# First day of every month at midnight
0 0 1 * * /path/to/chronomover --source "..." --destination "..."
```

To view your cron jobs:
```bash
crontab -l
```

### Testing Before Scheduling

Before setting up automatic scheduling, verify your configuration:

1. Run the command manually with your actual directories
2. Verify the files are moved correctly
3. Check that the folder structure is preserved as expected
4. Verify empty directories are deleted as expected (or use `--keep-empty-folders` to preserve them)
5. Test the batch file/script if using one

### Logging Output

Redirect output to a log file for troubleshooting and monitoring:

**Windows:**
```bash
chronomover.exe --source "C:\Notes" --destination "C:\Archive" > log.txt 2>&1
```

**macOS/Linux:**
```bash
chronomover --source "$HOME/Notes" --destination "$HOME/Archive" > log.txt 2>&1
```

**In a batch file with timestamps (Windows):**
```batch
@echo off
echo Running at %date% %time% >> C:\logs\chronomover.log
cd /d "C:\Tools\chronomover"
chronomover.exe --source "C:\Notes" --destination "C:\Archive" >> C:\logs\chronomover.log 2>&1
```

## Troubleshooting

### Build Issues

**"cargo: command not found"**
- Rust is not installed or not in your PATH
- Install Rust from https://rustup.rs/
- Restart your terminal after installation

**Build fails with dependency errors**
- Try updating Rust: `rustup update`
- Clean the build directory: `cargo clean`
- Rebuild: `cargo build --release`

### Runtime Issues

**"Invalid arguments" or "Missing required arguments"**
- Ensure you provide both `--source` and `--destination`
- Check that paths are enclosed in quotes if they contain spaces
- Use `--help` to see all available options

**"Source directory does not exist"**
- Verify the path is correct and the directory exists
- Check for typos in the path
- Use absolute paths instead of relative paths

**"Failed to move file"**
- Ensure you have read permissions for source directory
- Ensure you have write permissions for destination directory
- Check if files are open in another program
- Verify there's enough disk space

**"Permission denied" errors**
- Run with administrator/sudo privileges if necessary
- Check folder permissions
- Ensure antivirus isn't blocking the operation

**Files are moved but not grouped correctly**
- Verify the `--group-by` option is specified correctly
- Check that file timestamps are what you expect
- Use `--dry-run` to preview the grouping before moving

**No files are being moved**
- Check filters: `--older-than` and `--previous-period-only` might be excluding everything
- Use `--dry-run` to see what files are being detected
- Remember: `--previous-period-only` only works with `--group-by`
- Verify the source directory contains files

**Week numbers seem wrong**
- ChronoMover uses ISO 8601 week numbering
- Weeks start on Monday
- Week 1 is the first week containing a Thursday
- This may differ from your calendar app

**Ignored paths warning appears**
- Check that paths in `--ignored-paths` are absolute (full paths)
- Verify paths exist on disk (warnings are logged for non-existent paths)
- Ensure paths are comma-separated without spaces (unless part of the path)
- Use quotes around paths with spaces

**Unexpected files are still being moved (ignored paths not working)**
- Verify you're using absolute paths, not relative paths
- Check that the ignored path is a parent of the files you want to skip
- Ensure there are no typos in the ignored paths
- Use `--dry-run` to verify behavior before moving

**No files found when using depth limits**
- Check that your depth limits make sense (`min-depth` ≤ `max-depth`)
- Remember: depth 0 is the root, depth 1 is immediate subdirectories
- Use `--dry-run` to see which files are being detected
- Adjust depth values based on your directory structure

**Infinite loop or very slow when following symlinks**
- You likely have circular symbolic links in your directory structure
- Stop the operation (Ctrl+C)
- Run without `--follow-symbolic-links` (default behavior)
- Or use `--max-depth` to limit how deep the traversal goes
- Check your filesystem for circular symlink references

**Empty folders are not being deleted**
- Empty folder cleanup is skipped during `--dry-run` mode
- Run without `--dry-run` to actually delete empty folders
- Check that you're not using `--keep-empty-folders` flag
- Verify you have write permissions to delete directories

**Empty folders are being deleted but I want to keep them**
- Use `--keep-empty-folders` flag to preserve folder structure
- Example: `chronomover --source "C:\Notes" --destination "C:\Archive" --keep-empty-folders`
- This is useful for preserving organizational structure or template folders

### Task Scheduler / Automation Issues (Windows)

**Task shows error in Task Scheduler**
- Check the task's "History" tab for detailed error messages
- Verify the batch file path is correct in the Actions tab
- Ensure paths in the batch file use quotes for spaces
- Try running the batch file manually first
- Check that the executable path is correct

**Task runs but files aren't moved**
- Check the "Last Run Result" column
- Add logging to your batch file:
  ```batch
  @echo off
  echo Running at %date% %time% >> C:\logs\chronomover.log
  cd /d "C:\Tools\chronomover"
  chronomover.exe --source "C:\Notes" --destination "C:\Archive" >> C:\logs\chronomover.log 2>&1
  ```

### Platform-Specific Issues

**macOS: "Permission denied" when running executable**
- Make the file executable: `chmod +x chronomover`
- Grant permissions: System Preferences → Security & Privacy

**macOS: launchd agent not running**
- Check for syntax errors in the plist file
- View logs: `launchctl list | grep chronomover`
- Check system logs: Console app → filter for "chronomover"

**Linux: Cron job not running**
- Ensure the executable path is absolute
- Check cron logs: `grep CRON /var/log/syslog` (Debian/Ubuntu) or `/var/log/cron` (RedHat/CentOS)
- Make sure the file is executable: `chmod +x chronomover`

## Development Commands

If you're developing or modifying ChronoMover:

### Format Code
```bash
cargo fmt
```

### Check for Compilation Errors
```bash
cargo check
```

### Run Linter (Clippy)
```bash
cargo clippy
```

### Run Tests
```bash
cargo test
```

### Build for Release
```bash
cargo build --release
```

### Build for Multiple Platforms

**From Windows (for Windows):**
```bash
cargo build --release
```

**Cross-compilation** requires additional setup. See the Rust cross-compilation documentation for building Windows binaries on Linux/macOS, or vice versa.

### Exit Codes

The application returns standard exit codes:

- **0** - Success (all operations completed successfully)
- **1** - Error (check console output for details)

These are useful when running ChronoMover in scripts or automated workflows.

## Contributing

If you'd like to contribute to ChronoMover:

1. Fork the repository
2. Create a feature branch: `git checkout -b feature/my-new-feature`
3. Make your changes and commit: `git commit -am 'Add new feature'`
4. Push to the branch: `git push origin feature/my-new-feature`
5. Submit a pull request

Please ensure:
- Code follows Rust conventions
- `cargo fmt` has been run
- `cargo clippy` shows no warnings
- All tests pass: `cargo test`

## Architecture Notes

The application is organized into several modules:

- **`src/main.rs`** - Entry point and main execution flow
- **`src/model.rs`** - Data structures, argument parsing, validation
- **`src/file.rs`** - File operations, filtering, moving, cleanup
- **`src/date.rs`** - Date/time utilities, period calculations, timestamp handling
- **`src/log_macro.rs`** - Logging utilities

### Key Dependencies

- **chrono** - Date/time handling, ISO week calculations
- **walkdir** - Recursive directory traversal
- **clap** - Command-line argument parsing
- **color_eyre** - Error handling with context
- **humantime** - Parse human-readable durations

## License

This project is provided as-is for personal use.

## Support

For issues or questions:
- Report bugs on the [GitHub Issues page](https://github.com/SecretX33/ChronoMover/issues)
- Check the [main README](README.md) for basic usage
- Review this document for advanced topics

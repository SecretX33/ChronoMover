# ChronoMover

[![CI](https://github.com/SecretX33/ChronoMover/actions/workflows/build-and-release.yml/badge.svg)](https://github.com/SecretX33/ChronoMover/actions/workflows/build-and-release.yml)
[![GitHub release (latest by date)](https://img.shields.io/github/v/release/SecretX33/ChronoMover)](https://github.com/SecretX33/ChronoMover/releases/latest)
[![GitHub License](https://img.shields.io/github/license/SecretX33/ChronoMover)](https://github.com/SecretX33/ChronoMover/blob/master/LICENSE)
[![Rust Version](https://img.shields.io/badge/rust-stable-brightgreen.svg)](https://www.rust-lang.org/)

ChronoMover is a fast, efficient file organization utility written in Rust that automatically archives files based on their timestamps. Organize your workspace by moving files to an archive folder with flexible grouping strategies (week, month, year, etc.) while preserving folder structure.

## Features

- 🕒 Archive files based on their age (created, modified, or accessed time)
- 📁 Flexible time-based grouping (week, biweekly, month, trimester, quadrimester, semester, year)
- 🛡️ Preserves folder structure in the archive
- 📝 Dry run mode to preview changes before moving
- 🧹 Optional cleanup of empty folders after archiving
- 🔍 Smart filtering (move only previous periods, older than specific dates)
- 🌐 Cross-platform (Windows, macOS, Linux)

## Download

ChronoMover is available for Windows, Linux, and macOS.

Get the latest version [here](https://github.com/SecretX33/ChronoMover/releases/latest). Want an older version? Check all releases [here](https://github.com/SecretX33/ChronoMover/releases).

## Usage

```bash
chronomover --source <PATH> --destination <PATH> [OPTIONS]
```

### Required Arguments

- `-s, --source <PATH>`: Folder containing files to organize
- `-d, --destination <PATH>`: Where to move files

### Optional Arguments

- `-g, --group-by <STRATEGY>`: Group files by time period (week, biweekly, month, trimester, quadrimester, semester, year)
- `--file-date-types <TYPES>`: Specify which timestamps to check. You can use full names (created, modified, accessed) or first letters (c, m, a) [default: created,modified]
- `--previous-period-only`: Only move files from previous periods (excludes current period, requires --group-by)
- `--older-than <TIME>`: Only move files older than specified time (e.g., "30d", "1y", "2w3d")
- `--dry-run`: Preview what would be moved without actually moving [default: false]

### Time Format

The time format for `--older-than` supports human-readable formats:
- `1s`, `1sec` - 1 second
- `2m`, `2min` - 2 minutes
- `3h`, `3hr` - 3 hours
- `4d`, `4days` - 4 days
- `5w`, `5week` - 5 weeks
- `6M`, `6month` - 6 months
- `7y`, `7year` - 7 years

You can combine them: `1y6M` (1 year and 6 months), `2w3d` (2 weeks and 3 days), etc.

## Examples

#### Preview changes before moving (dry run)
```bash
chronomover --source "C:/Notes" --destination "C:/Archive" --dry-run
```

#### Move all files while preserving folder structure
```bash
chronomover --source "C:/Notes" --destination "C:/Archive"
```

#### Group files by month
```bash
chronomover --source "C:/Notes" --destination "C:/Archive" --group-by month
```

#### Move only files older than 30 days
```bash
chronomover --source "C:/Notes" --destination "C:/Archive" --older-than 30d
```

#### Move only previous week's files, excluding current week
```bash
chronomover --source "C:/Notes" --destination "C:/Archive" --group-by week --previous-period-only
```

#### Combine filters: group by week, previous periods only, older than 90 days
```bash
chronomover --source "C:/Notes" --destination "C:/Archive" --group-by week --previous-period-only --older-than 90d
```

#### Use specific timestamp types
```bash
chronomover --source "C:/Notes" --destination "C:/Archive" --file-date-types "modified,accessed"
```

**More examples and advanced usage →** See [ADVANCED_README.md](ADVANCED_README.md)

## Available Grouping Strategies

- **week** - Weekly folders using ISO week numbering (e.g., `2025-W49`)
- **biweekly** - Bi-weekly folders (e.g., `2025-BW01` through `2025-BW26`)
- **month** - Monthly folders (e.g., `2025-11`)
- **trimester** - Quarterly folders (e.g., `2025-Q1` through `2025-Q4`)
- **quadrimester** - 4-month periods (e.g., `2025-QD1` through `2025-QD3`)
- **semester** - Half-year folders (e.g., `2025-H1`, `2025-H2`)
- **year** - Yearly folders (e.g., `2025`)

For detailed format examples → See [ADVANCED_README.md - Grouping Strategies](ADVANCED_README.md#grouping-strategies)

## Safety Notes

- By default, ALL files are moved unless you use `--previous-period-only` or `--older-than`
- Always use `--dry-run` first to preview changes
- File timestamps depend on filesystem and OS support
- When multiple `--file-date-types` are specified, the most recent timestamp is used

## Troubleshooting

### "Source directory does not exist"
- Check that the path is correct and the folder exists
- Make sure to use quotes around paths with spaces

### "Failed to move file"
- Ensure you have permission to read from source and write to destination
- Close any programs that might be using the files
- Check that you have enough disk space

### Windows security warning when running
- Windows may block downloads from unknown sources
- Right-click the file → Properties → Check "Unblock" → OK
- Or click "More info" → "Run anyway" when the warning appears

### Files not being moved
- Try with `--dry-run` first to see what's being detected
- Check that filters (`--older-than`, `--previous-period-only`) aren't excluding everything
- Remember: `--previous-period-only` only works with `--group-by`

**More troubleshooting?** See [ADVANCED_README.md - Troubleshooting](ADVANCED_README.md#troubleshooting)

## Building from Source

- Install [Rust](https://www.rust-lang.org/tools/install)
- Build the binary by executing this command, the compiled file will be in the `target/[debug|release]` folder

```shell
# For development build
cargo build

# For release (optimized) build
cargo build --release
```

**For more details →** See [ADVANCED_README.md - Building from Source](ADVANCED_README.md#building-from-source)

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request with your changes, or open an Issue to request new features or report bugs.

## License

This project is licensed under the AGPL 3.0 license. See the [LICENSE](LICENSE) file for details.

# Changelog

All notable changes to this project will be documented in this file.

## [0.2.0] - 2026-04-14

### Added
- **Help popup** — press `?` to view all keyboard shortcuts
- **Search filter** — press `/` to filter Commands and Projects tabs by keyword
- **CSV export** — press `e` to export current tab data to `/tmp/rtk-tui-*.csv`
- **24-hour sparkline** — hourly token savings trend with time axis (`-24h` to `now`)
- **Time axis labels** — both 24h and 30-day sparklines now show axis ticks
- **Empty state** — friendly guide when no RTK data exists
- **Efficiency meter** — color-coded progress bar (green/yellow/red)
- **Summary KPIs** — input/output tokens, exec time with average, weighted savings %
- 11 new tests for truncate, duration, sparkline, weighted savings

### Fixed
- Data accuracy: use weighted savings (`SUM(saved)/SUM(input)*100`) matching `rtk gain`
- Number format: use K/M suffix (`1.2M`, `59.2K`) matching RTK output
- Column alignment: truncate long command names in Recent Commands
- Timezone: use `localtime` in SQLite queries to match local clock
- Sparkline rendering: stretch data points to fill terminal width

### Changed
- Dashboard layout: Summary + 24h sparkline + 30-day sparkline + Recent Commands
- Upgraded to Rust edition 2024, rust-version 1.92.0

## [0.1.0] - 2026-04-14

### Added
- Initial release
- 4 tabs: Dashboard, History, Commands, Projects
- Real-time SQLite read-only dashboard with auto-refresh
- Event-driven architecture with dirty-flag rendering
- 30-day sparkline chart
- Daily/weekly/monthly history tables
- Top commands ranking
- Per-project savings breakdown
- Panic-safe terminal restore
- GitHub CI and release workflows

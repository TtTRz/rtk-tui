# Changelog

All notable changes to this project will be documented in this file.

## [0.4.0] - 2026-04-16

### Changed
- Dashboard summary redesigned into two cards:
  - **Overview** card for Saved / Efficiency / Trend
  - **Details** card for Commands / Input / Output / Total time / Avg time
- 24-hour and 30-day sparklines now render side by side when terminal width allows, with cleaner axis labels
- Recent Commands redesigned as a lighter event table:
  - Wide terminals show `Command / Saved / Exec / Time`
  - Medium terminals show `Command / Saved / Exec`
  - Narrow terminals show `Command / Saved`
- Dashboard spacing, alignment, and information hierarchy were tightened for easier scanning in terminal layouts

### Fixed
- 24-hour sparkline axis now uses hour-aligned labels and keeps `now` as the rightmost marker
- 30-day sparkline axis now follows a more regular weekly cadence
- Summary and Recent Commands alignment improved to reduce visual noise

## [0.3.0] - 2026-04-15

### Added
- **Buddy companion** — ASCII pet on the Dashboard, inspired by Claude Code's buddy system
  - 7 species: Llama, Cat, Duck, Blob, Robot, Penguin, Ghost
  - Deterministic species from DB path hash (FNV-1a), or choose with `--buddy <name>`
  - 5 moods (Ecstatic/Happy/Content/Sleepy/Worried) driven by token savings data
  - Action state machine: Idle, WalkLeft, WalkRight, Bounce
  - Speech bubbles with mood-specific messages (30 total, 6 per mood)
  - Mood-colored sprites with eye expressions (✦/·/°/-/×)
  - Responsive: only shown when terminal ≥ 75 columns wide
- `--buddy` CLI flag to select species at startup
- 13 new buddy tests (species, mood, animation, bubble, walk, bounce)

### Changed
- Sprite frames expanded to 6 lines for consistent rendering across all species
- Dashboard layout: Summary splits 60/40 for KPI + Buddy panel

### Fixed
- Bounce animation no longer hides Llama ears (remove bubble line instead of sprite line)

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

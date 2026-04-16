<div align="center">

# 🖥️ RTK-TUI

### Real-time terminal dashboard for RTK token savings

<br>

[![crates.io](https://img.shields.io/crates/v/rtk-tui.svg?style=flat-square&logo=rust)](https://crates.io/crates/rtk-tui)
[![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg?style=flat-square)](LICENSE)
[![Rust 1.92+](https://img.shields.io/badge/rust-1.92%2B-orange?style=flat-square&logo=rust)](https://www.rust-lang.org/)
[![RTK](https://img.shields.io/badge/powered%20by-RTK-brightgreen?style=flat-square)](https://github.com/rtk-ai/rtk)

<br>

[RTK](https://github.com/rtk-ai/rtk) saves 60–90% of LLM tokens by filtering CLI output.

**RTK-TUI** — Real-time terminal dashboard for RTK token savings.

<br>

[English](README.md) | [中文](README.zh-CN.md)

<br>

</div>

## 🔍 Preview

```
┌ RTK Token Savings ──────────────────────────────────────────────────────────┐
│  1 Dashboard    2 History    3 Commands    4 Projects                        │
├ Overview ─────────────────────────────┬ Buddy ──────────────────────────────┤
│  Saved:       5.7M                    │   .-----------.                      │
│  Efficiency:  █████████████████ 93.0% │   | Keep it up! |                    │
│  Trend:       ↑ +18% vs 7d avg (210K) │   `-----------'                      │
├ Details ──────────────────────────────┤         \                            │
│  Commands:    368                     │        /)  /)                        │
│  Input:       324.1K                  │      ( ·   · )                       │
│  Output:      23.1K                   │      ((  ᵕ  ))                       │
│  Total time:  9m22s                   │     __| --- |__                      │
│  Avg time:    1.5s                    │                                      │
├ Last 24 Hours · 45.2K · Pk 4.1K ─────┬ Last 30 Days · Pk 301K ─────────────┤
│ ▁▂▃▅▇█▇▅▃▂▁▂▃▅▇█▇▅▃▂▁▂▃▅▇█▆▅▃▂▁▂▃▅▇█ │ ░▂▃▅▇█▆▅▃▂▁▂▃▅▇█▇▅▃▂▁▁▂▃▅▆▇█▇▆▅▂▃▅█ │
│ 12:00      16:00      20:00      now │ 03/19        03/26        now       │
├ Recent Commands ────────────────────────────────────────────────────────────┤
│  Command                               Saved    Exec   Time                  │
│  rtk cargo clippy                      4.5K    3.5s  18:41                  │
│  rtk git status                          800    1.2s  18:42                  │
└─────────────────────────────────────────────────────────────────────────────┘
```

## ⚡ Quick Start

```bash
cargo install rtk-tui    # from crates.io
rtk-tui                  # that's it
```

<details>
<summary><b>Build from source</b></summary>

```bash
git clone https://github.com/TtTRz/rtk-tui.git
cd rtk-tui
cargo build --release
# → target/release/rtk-tui
```

</details>

> **Requires:** [RTK](https://github.com/rtk-ai/rtk) installed and used (to generate tracking data) · Rust 1.92+

## 📖 Usage

```bash
rtk-tui                     # auto-detect RTK database
rtk-tui --db /path/to.db    # specify database path
rtk-tui --refresh 5         # refresh every 5s (default: 1s)
rtk-tui --buddy llama       # choose your buddy species
```

### Buddy Species

The dashboard includes an ASCII companion that reacts to your token savings.
Species is auto-assigned from your DB path hash, or pick one with `--buddy`:

Available species: `llama` · `cat` · `duck` · `blob` · `robot` · `penguin` · `ghost`

Buddy mood is driven by your savings data: **Ecstatic** (50K+ saved/24h) → **Happy** (≥80% efficiency) → **Content** (≥50%) → **Worried** (<50%) → **Sleepy** (no activity).

## ⌨️ Keyboard

<table>
<tr>
<td>

**Navigation**

| Key | Action |
|:----|:-------|
| `1` `2` `3` `4` | Switch tabs |
| `Tab` | Next tab |
| `j` `↓` | Scroll down |
| `k` `↑` | Scroll up |
| `r` | Force refresh |
| `q` `Esc` | Quit |

</td>
<td>

**History Tab**

| Key | Action |
|:----|:-------|
| `d` | Daily view |
| `w` | Weekly view |
| `m` | Monthly view |

**Other**

| Key | Action |
|:----|:-------|
| `?` | Help popup |
| `/` | Search (Commands / Projects) |
| `e` | Export current tab as CSV |

</td>
</tr>
</table>

## 📊 Tabs

| # | Tab | Description |
|:-:|:----|:------------|
| 1 | **Dashboard** | Overview / Details cards · efficiency meter · buddy companion · side-by-side 24h & 30-day sparklines · recent commands |
| 2 | **History** | Daily / weekly / monthly table — toggle with `d` `w` `m` |
| 3 | **Commands** | Top commands ranked by total tokens saved · searchable with `/` |
| 4 | **Projects** | Per-project savings breakdown · searchable with `/` |

## ✨ Features

- **Live dashboard** — auto-refreshes every second, shows real-time token savings
- **Buddy companion** — ASCII pet that walks, bounces, and reacts to your savings (7 species, 5 moods)
- **24h & 30-day sparklines** — hourly and daily trends with cleaner axis labels, rendered side by side on wide terminals
- **Dashboard cards** — split Overview / Details cards for faster scanning of key metrics and secondary stats
- **Efficiency meter** — visual progress bar with color-coded thresholds
- **Search & filter** — press `/` to filter Commands and Projects by keyword
- **CSV export** — press `e` to export current tab data to `/tmp/rtk-tui-*.csv`
- **Help popup** — press `?` for a quick keyboard reference
- **Empty state** — friendly guide when no RTK data exists yet
- **Read-only & secure** — `SQLITE_OPEN_READ_ONLY`, parameterized queries, escape injection protection

## 🏗️ Architecture

```
                        ┌─────────────────────────────────────┐
                        │            Main Thread              │
                        │                                     │
┌──────────────┐        │  ┌───────────┐    ┌──────────────┐  │
│ Input Thread │──event─┤  │           │    │              │  │
│ (crossterm)  │        │  │  App Loop │───▶│  ratatui     │──┤──▶ Terminal
│              │        │  │           │    │  render       │  │
│ Tick Thread  │──tick──┤  │     ▲     │    └──────────────┘  │
│ (refresh)    │        │  │     │     │                      │
└──────────────┘        │  │  DataCache │◀── SQLite (RO)      │
                        │  └───────────┘                      │
                        └─────────────────────────────────────┘
```

<details>
<summary><b>Design choices</b></summary>

- **Event-driven** — 2 background threads → `mpsc::channel` → main loop blocks on `recv()`, redraws only when dirty
- **Cached queries** — all DB reads happen once per tick into `DataCache`, render functions never touch the DB
- **Read-only** — `SQLITE_OPEN_READ_ONLY` + `PRAGMA query_only` — zero risk to your data
- **Zero coupling** — no dependency on the RTK crate, communicates through the SQLite schema only
- **Secure** — terminal escape injection protection, GLOB metacharacter escaping, parameterized queries

</details>

## 📂 Database Location

RTK-TUI finds RTK's database automatically:

| # | Source | Example |
|:-:|--------|---------|
| 1 | `--db` flag | `rtk-tui --db ~/my.db` |
| 2 | `RTK_DB_PATH` env | `export RTK_DB_PATH=~/my.db` |
| 3 | Platform default | ↓ |

| Platform | Default Path |
|:---------|:-------------|
| macOS | `~/Library/Application Support/rtk/history.db` |
| Linux | `~/.local/share/rtk/history.db` |
| Windows | `%APPDATA%\rtk\history.db` |

## 🗂️ Project Structure

```
src/
├── main.rs            Entry point · CLI args · panic-safe terminal restore
├── app.rs             Event loop · state machine · data cache · dirty-flag
├── db.rs              Read-only SQLite queries · prepare_cached · GLOB escape
├── event.rs           Input thread + tick thread → mpsc channel
├── export.rs          CSV export for all tabs
├── buddy/
│   ├── mod.rs         BuddyState · public API · species/mood/animation coordination
│   ├── species.rs     7 species · ASCII sprite frames (3 frames × 6 lines each)
│   ├── mood.rs        5 moods · eye chars · message pools (30 messages)
│   ├── animation.rs   Action state machine · timing · PRNG · bubble lifecycle
│   └── render.rs      Sprite positioning · speech bubble layout · mood coloring
└── ui/
    ├── mod.rs         Tab bar · status bar · help popup · empty state · format helpers
    ├── dashboard.rs   Summary KPIs · buddy panel · efficiency meter · sparklines
    ├── history.rs     Daily / weekly / monthly tables · scroll indicator
    ├── commands.rs    Top commands ranking · search filter
    └── projects.rs    Per-project breakdown · search filter
```

## 📄 License

[MIT](LICENSE)

<div align="center">

[English](README.md) | [中文](README.zh-CN.md)

# rtk-tui

**Real-time terminal dashboard for your RTK token savings**

[![Rust](https://img.shields.io/badge/rust-stable-orange?logo=rust)](https://www.rust-lang.org/)
[![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)
[![RTK](https://img.shields.io/badge/powered%20by-RTK-brightgreen)](https://github.com/rtk-ai/rtk)

[RTK](https://github.com/rtk-ai/rtk) saves 60–90% of LLM tokens by filtering CLI output.<br>
**rtk-tui** turns that data into a live dashboard you can watch while you code.

</div>

---

## Preview

```
┌ RTK Token Savings ──────────────────────────────────────────────────────┐
│  1 Dashboard   2 History   3 Commands   4 Projects                      │
├─────────────────┬─────────────────┬─────────────────┬───────────────────┤
│ Tokens Saved    │ Avg Savings     │ Commands        │ Total Time        │
│                 │                 │                 │                   │
│ 1,247,832       │ 78.3%           │ 3,412           │ 42.7s             │
├─────────────────┴─────────────────┴─────────────────┴───────────────────┤
│ Last 30 Days — Tokens Saved                                             │
│ ▂▃▅▇█▆▅▃▂▁▂▃▅▇█▇▅▃▂▁▁▂▃▅▆▇█▇▆▅                                       │
├─────────────────────────────────────────────────────────────────────────┤
│ Recent Commands                                                         │
│ 2026-04-14 18:42   rtk git status          +1,200 (82%)                │
│ 2026-04-14 18:41   rtk cargo test          +4,500 (90%)                │
│ 2026-04-14 18:40   rtk git log             +1,600 (80%)                │
└─────────────────────────────────────────────────────────────────────────┘
```

## Quick Start

```bash
# Build and install
cargo install --path .

# Run (auto-detects RTK database)
rtk-tui
```

## Installation

**From source:**

```bash
git clone https://github.com/TtTRz/rtk-tui.git
cd rtk-tui
cargo build --release
# → target/release/rtk-tui
```

**Prerequisites:** [RTK](https://github.com/rtk-ai/rtk) installed and used (to generate tracking data), Rust 1.70+.

## Usage

```bash
rtk-tui                     # auto-detect RTK database
rtk-tui --db /path/to.db    # specify database path
rtk-tui --refresh 5         # refresh every 5s (default: 1s)
```

## Keyboard Shortcuts

```
Navigation                    History Tab
─────────────────────────     ─────────────────
1 2 3 4    Switch tabs        d   Daily view
Tab        Next tab           w   Weekly view
j / ↓      Scroll down        m   Monthly view
k / ↑      Scroll up

General
─────────────────────────
r          Force refresh
q / Esc    Quit
```

## Tabs

| Tab | What you see |
|:---:|:-------------|
| **1 Dashboard** | Metric cards (saved, avg %, commands, time) · 30-day sparkline · recent commands |
| **2 History** | Daily / weekly / monthly table with `d`/`w`/`m` toggle |
| **3 Commands** | Top commands ranked by total tokens saved |
| **4 Projects** | Per-project savings breakdown |

## How It Works

```
┌──────────────┐     ┌────────────────┐     ┌──────────────┐
│  Input Thread │────▶│                │     │              │
│  (crossterm)  │     │  mpsc channel  │────▶│   App Loop   │──▶ ratatui render
│               │     │                │     │              │
│  Tick Thread  │────▶│  Key/Tick/     │     │  DataCache   │◀── SQLite (read-only)
│  (refresh)    │     │  Resize        │     │              │
└──────────────┘     └────────────────┘     └──────────────┘
```

**Key design choices:**

- **Event-driven** — background threads send events via `mpsc::channel`, main loop blocks on `recv()`, redraws only when dirty
- **Cached queries** — all DB reads happen once per tick into a `DataCache` struct, render functions never touch the DB
- **Read-only** — `SQLITE_OPEN_READ_ONLY` + `PRAGMA query_only`, zero risk to your data
- **Zero coupling** — no dependency on the RTK crate, communicates through the SQLite schema only

## Database Location

rtk-tui finds RTK's tracking database automatically:

| Priority | Source |
|:--------:|--------|
| 1 | `--db` CLI flag |
| 2 | `RTK_DB_PATH` env var |
| 3 | Platform default ↓ |

| Platform | Path |
|----------|------|
| macOS | `~/Library/Application Support/rtk/history.db` |
| Linux | `~/.local/share/rtk/history.db` |
| Windows | `%APPDATA%\rtk\history.db` |

## Project Structure

```
src/
├── main.rs          CLI entry + panic-safe terminal restore
├── app.rs           Event loop, state machine, data cache
├── db.rs            SQLite queries (parameterized, prepare_cached)
├── event.rs         Input thread + tick thread → channel
└── ui/
    ├── mod.rs       Tab bar, error status bar, format helpers
    ├── dashboard.rs Summary cards + sparkline + recent commands
    ├── history.rs   Daily / weekly / monthly tables
    ├── commands.rs  Top commands ranking table
    └── projects.rs  Per-project breakdown table
```

## License

[MIT](LICENSE)

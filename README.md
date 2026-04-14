<div align="center">

# 🖥️ RTK-TUI

### Real-time terminal dashboard for your RTK token savings

<br>

[![crates.io](https://img.shields.io/crates/v/RTK-TUI.svg?style=flat-square&logo=rust)](https://crates.io/crates/RTK-TUI)
[![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg?style=flat-square)](LICENSE)
[![Rust 1.92+](https://img.shields.io/badge/rust-1.92%2B-orange?style=flat-square&logo=rust)](https://www.rust-lang.org/)
[![RTK](https://img.shields.io/badge/powered%20by-RTK-brightgreen?style=flat-square)](https://github.com/rtk-ai/rtk)

<br>

[RTK](https://github.com/rtk-ai/rtk) saves 60–90% of LLM tokens by filtering CLI output.

**RTK-TUI** turns that invisible work into a dashboard you can actually see.

<br>

[English](README.md) | [中文](README.zh-CN.md)

<br>

</div>

## 🔍 Preview

```
┌ RTK Token Savings ──────────────────────────────────────────────────────────┐
│  1 Dashboard    2 History    3 Commands    4 Projects                        │
├──────────────────┬──────────────────┬──────────────────┬────────────────────┤
│  Tokens Saved    │  Avg Savings     │  Commands        │  Total Time        │
│                  │                  │                  │                    │
│  1,247,832    ▲  │  78.3%           │  3,412           │  42.7s             │
├──────────────────┴──────────────────┴──────────────────┴────────────────────┤
│  Last 30 Days — Tokens Saved                                                │
│                                                                             │
│  ▂▃▅▇█▆▅▃▂▁▂▃▅▇█▇▅▃▂▁▁▂▃▅▆▇█▇▆▅                                          │
│                                                                             │
├─────────────────────────────────────────────────────────────────────────────┤
│  Recent Commands                                                            │
│                                                                             │
│  2026-04-14 18:42    rtk git status              +1,200  (82%)              │
│  2026-04-14 18:41    rtk cargo test              +4,500  (90%)              │
│  2026-04-14 18:40    rtk git log                 +1,600  (80%)              │
│  2026-04-14 18:39    rtk cargo clippy            +3,200  (85%)              │
│  2026-04-14 18:38    rtk git diff                +2,100  (76%)              │
└─────────────────────────────────────────────────────────────────────────────┘
```

## ⚡ Quick Start

```bash
cargo install RTK-TUI    # from crates.io
RTK-TUI                  # that's it
```

<details>
<summary><b>Build from source</b></summary>

```bash
git clone https://github.com/TtTRz/RTK-TUI.git
cd RTK-TUI
cargo build --release
# → target/release/RTK-TUI
```

</details>

> **Requires:** [RTK](https://github.com/rtk-ai/rtk) installed and used (to generate tracking data) · Rust 1.92+

## 📖 Usage

```bash
RTK-TUI                     # auto-detect RTK database
RTK-TUI --db /path/to.db    # specify database path
RTK-TUI --refresh 5         # refresh every 5s (default: 1s)
```

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

</td>
</tr>
</table>

## 📊 Tabs

| # | Tab | Description |
|:-:|:----|:------------|
| 1 | **Dashboard** | 4 metric cards · 30-day sparkline · recent commands |
| 2 | **History** | Daily / weekly / monthly table — toggle with `d` `w` `m` |
| 3 | **Commands** | Top commands ranked by total tokens saved |
| 4 | **Projects** | Per-project savings breakdown |

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
| 1 | `--db` flag | `RTK-TUI --db ~/my.db` |
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
└── ui/
    ├── mod.rs         Tab bar · error status bar · sanitize · format helpers
    ├── dashboard.rs   Summary cards · sparkline · recent commands
    ├── history.rs     Daily / weekly / monthly tables · scroll indicator
    ├── commands.rs    Top commands ranking
    └── projects.rs    Per-project breakdown
```

## 📄 License

[MIT](LICENSE)

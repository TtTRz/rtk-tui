<div align="center">

[English](README.md) | [中文](README.zh-CN.md)

<br>

# 🖥️ RTK-TUI

### RTK Token 节省量的实时终端仪表盘

<br>

[![crates.io](https://img.shields.io/crates/v/rtk-tui.svg?style=flat-square&logo=rust)](https://crates.io/crates/rtk-tui)
[![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg?style=flat-square)](LICENSE)
[![Rust 1.92+](https://img.shields.io/badge/rust-1.92%2B-orange?style=flat-square&logo=rust)](https://www.rust-lang.org/)
[![RTK](https://img.shields.io/badge/powered%20by-RTK-brightgreen?style=flat-square)](https://github.com/rtk-ai/rtk)

<br>

[RTK](https://github.com/rtk-ai/rtk) 通过过滤 CLI 输出，为 LLM 节省 60–90% 的 token。

**RTK-TUI** — RTK token 节省量的实时终端仪表盘。

<br>

</div>

## 🔍 预览

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

## ⚡ 快速开始

```bash
cargo install rtk-tui    # 从 crates.io 安装
rtk-tui                  # 搞定
```

<details>
<summary><b>从源码编译</b></summary>

```bash
git clone https://github.com/TtTRz/rtk-tui.git
cd rtk-tui
cargo build --release
# → target/release/rtk-tui
```

</details>

> **前置条件：** 已安装 [RTK](https://github.com/rtk-ai/rtk) 并使用过（需要有追踪数据） · Rust 1.92+

## 📖 使用

```bash
rtk-tui                     # 自动检测 RTK 数据库
rtk-tui --db /path/to.db    # 指定数据库路径
rtk-tui --refresh 5         # 每 5 秒刷新（默认 1 秒）
rtk-tui --buddy llama       # 选择你的 buddy 物种
```

### Buddy 物种

Dashboard 面板内置了一个 ASCII 电子宠物，会根据你的 token 节省数据做出反应。
物种默认由数据库路径哈希决定，也可以通过 `--buddy` 手动选择：

可选物种：`llama` · `cat` · `duck` · `blob` · `robot` · `penguin` · `ghost`

Buddy 的心情由节省数据驱动：**Ecstatic**（24h 节省 50K+）→ **Happy**（效率 ≥80%）→ **Content**（≥50%）→ **Worried**（<50%）→ **Sleepy**（无活动）。

## ⌨️ 快捷键

<table>
<tr>
<td>

**导航**

| 按键 | 操作 |
|:-----|:-----|
| `1` `2` `3` `4` | 切换标签页 |
| `Tab` | 下一个标签页 |
| `j` `↓` | 向下滚动 |
| `k` `↑` | 向上滚动 |
| `r` | 强制刷新 |
| `q` `Esc` | 退出 |

</td>
<td>

**History 标签页**

| 按键 | 操作 |
|:-----|:-----|
| `d` | 按日查看 |
| `w` | 按周查看 |
| `m` | 按月查看 |

**其他**

| 按键 | 操作 |
|:-----|:-----|
| `?` | 帮助弹窗 |
| `/` | 搜索（Commands / Projects） |
| `e` | 导出当前标签页为 CSV |

</td>
</tr>
</table>

## 📊 标签页

| # | 标签 | 说明 |
|:-:|:-----|:-----|
| 1 | **Dashboard** | Overview / Details 双卡 · 效率仪表 · Buddy 伙伴 · 并排的 24 小时与 30 天趋势图 · 最近命令 |
| 2 | **History** | 按日 / 周 / 月的明细表格 — `d` `w` `m` 切换 |
| 3 | **Commands** | 按节省 token 总量排名的命令列表 · 支持 `/` 搜索 |
| 4 | **Projects** | 按项目维度的节省量统计 · 支持 `/` 搜索 |

## ✨ 特性

- **实时仪表盘** — 每秒自动刷新，实时展示 token 节省量
- **Buddy 伙伴** — ASCII 电子宠物，会行走、跳跃并根据节省数据变化心情（7 种物种、5 种心情）
- **24 小时 & 30 天趋势图** — 逐小时和逐日 sparkline，时间刻度更整洁，宽终端下并排展示
- **Dashboard 双卡** — Overview / Details 分组展示核心指标和次级统计，终端里更易扫读
- **效率仪表** — 彩色进度条，按阈值变色（绿 / 黄 / 红）
- **搜索过滤** — 按 `/` 在 Commands 和 Projects 中按关键词过滤
- **CSV 导出** — 按 `e` 导出当前标签页数据到 `/tmp/rtk-tui-*.csv`
- **帮助弹窗** — 按 `?` 快速查看所有快捷键
- **空状态提示** — 无数据时显示友好引导
- **只读安全** — `SQLITE_OPEN_READ_ONLY`、参数化查询、转义注入防护

## 🏗️ 工作原理

```
                        ┌─────────────────────────────────────┐
                        │              主线程                  │
                        │                                     │
┌──────────────┐        │  ┌───────────┐    ┌──────────────┐  │
│  输入线程     │──事件──┤  │           │    │              │  │
│ (crossterm)  │        │  │  主循环    │───▶│  ratatui     │──┤──▶ 终端
│              │        │  │           │    │  渲染         │  │
│  定时线程     │──tick──┤  │     ▲     │    └──────────────┘  │
│ (refresh)    │        │  │     │     │                      │
└──────────────┘        │  │  数据缓存  │◀── SQLite（只读）    │
                        │  └───────────┘                      │
                        └─────────────────────────────────────┘
```

<details>
<summary><b>核心设计</b></summary>

- **事件驱动** — 2 个后台线程 → `mpsc::channel` → 主循环阻塞在 `recv()`，仅在状态变化时重绘
- **数据缓存** — 每次 tick 统一查询 DB 写入 `DataCache`，渲染函数只读缓存，不碰数据库
- **只读访问** — `SQLITE_OPEN_READ_ONLY` + `PRAGMA query_only`，对你的数据零风险
- **零耦合** — 不依赖 RTK crate，仅通过 SQLite schema 通信，独立安装
- **安全** — 终端转义注入防护、GLOB 元字符转义、参数化查询

</details>

## 📂 数据库位置

RTK-TUI 自动查找 RTK 的追踪数据库：

| # | 来源 | 示例 |
|:-:|------|------|
| 1 | `--db` 参数 | `rtk-tui --db ~/my.db` |
| 2 | `RTK_DB_PATH` 环境变量 | `export RTK_DB_PATH=~/my.db` |
| 3 | 平台默认路径 | ↓ |

| 平台 | 默认路径 |
|:-----|:---------|
| macOS | `~/Library/Application Support/rtk/history.db` |
| Linux | `~/.local/share/rtk/history.db` |
| Windows | `%APPDATA%\rtk\history.db` |

## 🗂️ 项目结构

```
src/
├── main.rs            入口 · CLI 参数 · panic 安全终端恢复
├── app.rs             事件循环 · 状态机 · 数据缓存 · dirty flag
├── db.rs              只读 SQLite 查询 · prepare_cached · GLOB 转义
├── event.rs           输入线程 + 定时线程 → mpsc channel
├── export.rs          CSV 导出
├── buddy/
│   ├── mod.rs         BuddyState · 公共 API · 物种/心情/动画协调
│   ├── species.rs     7 种物种 · ASCII 精灵帧（每种 3 帧 × 6 行）
│   ├── mood.rs        5 种心情 · 眼睛字符 · 消息池（30 条消息）
│   ├── animation.rs   行为状态机 · 时间控制 · PRNG · 气泡生命周期
│   └── render.rs      精灵定位 · 气泡布局 · 心情配色
└── ui/
    ├── mod.rs         标签栏 · 状态栏 · 帮助弹窗 · 空状态 · 格式化工具
    ├── dashboard.rs   Summary KPI · Buddy 面板 · 效率仪表 · sparklines
    ├── history.rs     日 / 周 / 月统计表格 · 滚动指示器
    ├── commands.rs    命令排行表格 · 搜索过滤
    └── projects.rs    项目维度统计表格 · 搜索过滤
```

## 📄 许可证

[MIT](LICENSE)

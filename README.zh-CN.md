<div align="center">

[English](README.md) | [中文](README.zh-CN.md)

# rtk-tui

**RTK Token 节省量的实时终端仪表盘**

[![Rust](https://img.shields.io/badge/rust-stable-orange?logo=rust)](https://www.rust-lang.org/)
[![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)
[![RTK](https://img.shields.io/badge/powered%20by-RTK-brightgreen)](https://github.com/rtk-ai/rtk)

[RTK](https://github.com/rtk-ai/rtk) 通过过滤 CLI 输出，为 LLM 节省 60–90% 的 token。<br>
**rtk-tui** 把这些数据变成一个实时终端仪表盘，边写代码边看省了多少。

</div>

---

## 预览

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

## 快速开始

```bash
# 编译安装
cargo install --path .

# 运行（自动检测 RTK 数据库）
rtk-tui
```

## 安装

**从源码编译：**

```bash
git clone https://github.com/TtTRz/rtk-tui.git
cd rtk-tui
cargo build --release
# → target/release/rtk-tui
```

**前置条件：** 已安装 [RTK](https://github.com/rtk-ai/rtk) 并使用过（需要有追踪数据），Rust 1.70+。

## 使用

```bash
rtk-tui                     # 自动检测 RTK 数据库
rtk-tui --db /path/to.db    # 指定数据库路径
rtk-tui --refresh 5         # 每 5 秒刷新（默认 1 秒）
```

## 快捷键

```
导航                          History 标签页
─────────────────────────     ─────────────────
1 2 3 4    切换标签页          d   按日查看
Tab        下一个标签页        w   按周查看
j / ↓      向下滚动            m   按月查看
k / ↑      向上滚动

通用
─────────────────────────
r          强制刷新
q / Esc    退出
```

## 标签页

| 标签 | 内容 |
|:----:|:-----|
| **1 Dashboard** | 指标卡片（节省量、平均节省率、命令数、耗时） · 30 天 sparkline · 最近命令 |
| **2 History** | 按日 / 周 / 月的明细表格，`d`/`w`/`m` 切换 |
| **3 Commands** | 按节省 token 总量排名的命令列表 |
| **4 Projects** | 按项目维度的节省量统计 |

## 工作原理

```
┌──────────────┐     ┌────────────────┐     ┌──────────────┐
│  输入线程     │────▶│                │     │              │
│  (crossterm)  │     │  mpsc channel  │────▶│   主循环      │──▶ ratatui 渲染
│               │     │                │     │              │
│  定时线程     │────▶│  按键/定时/     │     │  数据缓存     │◀── SQLite（只读）
│  (refresh)    │     │  窗口调整       │     │              │
└──────────────┘     └────────────────┘     └──────────────┘
```

**核心设计：**

- **事件驱动** — 后台线程通过 `mpsc::channel` 发送事件，主循环阻塞在 `recv()` 上，仅在状态变化时重绘（dirty flag）
- **数据缓存** — 每次 tick 统一查询 DB 写入 `DataCache`，渲染函数只读缓存，不碰数据库
- **只读访问** — `SQLITE_OPEN_READ_ONLY` + `PRAGMA query_only`，对你的数据零风险
- **零耦合** — 不依赖 RTK crate，仅通过 SQLite schema 通信，独立安装

## 数据库位置

rtk-tui 自动查找 RTK 的追踪数据库：

| 优先级 | 来源 |
|:------:|------|
| 1 | `--db` 命令行参数 |
| 2 | `RTK_DB_PATH` 环境变量 |
| 3 | 平台默认路径 ↓ |

| 平台 | 路径 |
|------|------|
| macOS | `~/Library/Application Support/rtk/history.db` |
| Linux | `~/.local/share/rtk/history.db` |
| Windows | `%APPDATA%\rtk\history.db` |

## 项目结构

```
src/
├── main.rs          入口 + panic 安全终端恢复
├── app.rs           事件循环、状态机、数据缓存
├── db.rs            SQLite 查询（参数化、prepare_cached）
├── event.rs         输入线程 + 定时线程 → channel
└── ui/
    ├── mod.rs       标签栏、错误状态栏、格式化工具
    ├── dashboard.rs 总览卡片 + sparkline + 最近命令
    ├── history.rs   日 / 周 / 月统计表格
    ├── commands.rs  命令排行表格
    └── projects.rs  项目维度统计表格
```

## 许可证

[MIT](LICENSE)

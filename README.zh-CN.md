<div align="center">

[English](README.md) | [中文](README.zh-CN.md)

<br>

# 🖥️ RTK-TUI

### RTK Token 节省量的实时终端仪表盘

<br>

[![crates.io](https://img.shields.io/crates/v/RTK-TUI.svg?style=flat-square&logo=rust)](https://crates.io/crates/RTK-TUI)
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

## ⚡ 快速开始

```bash
cargo install RTK-TUI    # 从 crates.io 安装
RTK-TUI                  # 搞定
```

<details>
<summary><b>从源码编译</b></summary>

```bash
git clone https://github.com/TtTRz/RTK-TUI.git
cd RTK-TUI
cargo build --release
# → target/release/RTK-TUI
```

</details>

> **前置条件：** 已安装 [RTK](https://github.com/rtk-ai/rtk) 并使用过（需要有追踪数据） · Rust 1.92+

## 📖 使用

```bash
RTK-TUI                     # 自动检测 RTK 数据库
RTK-TUI --db /path/to.db    # 指定数据库路径
RTK-TUI --refresh 5         # 每 5 秒刷新（默认 1 秒）
```

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

</td>
</tr>
</table>

## 📊 标签页

| # | 标签 | 说明 |
|:-:|:-----|:-----|
| 1 | **Dashboard** | 4 张指标卡片 · 30 天 sparkline · 最近命令 |
| 2 | **History** | 按日 / 周 / 月的明细表格 — `d` `w` `m` 切换 |
| 3 | **Commands** | 按节省 token 总量排名的命令列表 |
| 4 | **Projects** | 按项目维度的节省量统计 |

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
| 1 | `--db` 参数 | `RTK-TUI --db ~/my.db` |
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
└── ui/
    ├── mod.rs         标签栏 · 错误状态栏 · sanitize · 格式化工具
    ├── dashboard.rs   总览卡片 · sparkline · 最近命令
    ├── history.rs     日 / 周 / 月统计表格 · 滚动指示器
    ├── commands.rs    命令排行表格
    └── projects.rs    项目维度统计表格
```

## 📄 许可证

[MIT](LICENSE)

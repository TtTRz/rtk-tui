mod app;
mod buddy;
mod db;
mod event;
mod export;
mod ui;

use anyhow::{Context, Result};
use clap::Parser;
use std::path::PathBuf;

/// TUI dashboard for RTK token savings analytics
#[derive(Parser)]
#[command(name = "rtk-tui", version, about)]
struct Cli {
    /// Path to RTK tracking database (auto-detected if omitted)
    #[arg(long, value_name = "PATH")]
    db: Option<PathBuf>,

    /// Refresh interval in seconds
    #[arg(long, default_value = "1", value_name = "SECS")]
    refresh: u64,

    /// Choose your buddy species: llama, cat, duck, blob, robot, penguin, ghost
    #[arg(long, value_name = "SPECIES")]
    buddy: Option<String>,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    let db_path_str = cli
        .db
        .as_ref()
        .and_then(|p| p.to_str())
        .unwrap_or("default")
        .to_string();
    let db = db::Db::open(cli.db.as_ref().and_then(|p| p.to_str()))
        .context("Failed to open RTK tracking database")?;

    // Install panic hook to restore terminal before printing panic info
    let original_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        ratatui::restore();
        original_hook(info);
    }));

    let mut terminal = ratatui::init();
    let result =
        app::App::new(db, cli.refresh, &db_path_str, cli.buddy.as_deref()).run(&mut terminal);
    ratatui::restore();

    result
}

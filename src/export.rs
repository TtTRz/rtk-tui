use anyhow::{Context, Result};
use std::io::Write;
use std::path::PathBuf;

use crate::app::{App, HistoryView, Tab};

/// Export current tab data to a CSV file in the temp directory.
/// Returns the path of the exported file.
pub fn export_csv(app: &App) -> Result<PathBuf> {
    let tab_name = app.tab.title().to_lowercase();
    let timestamp = chrono::Local::now().format("%Y%m%d-%H%M%S");
    let filename = format!("rtk-tui-{tab_name}-{timestamp}.csv");
    let path = std::env::temp_dir().join(filename);

    let mut file = std::fs::File::create(&path)
        .with_context(|| format!("Cannot create {}", path.display()))?;

    match app.tab {
        Tab::Dashboard => write_dashboard(&mut file, app)?,
        Tab::History => match app.history_view {
            HistoryView::Daily => write_daily(&mut file, app)?,
            HistoryView::Weekly => write_weekly(&mut file, app)?,
            HistoryView::Monthly => write_monthly(&mut file, app)?,
        },
        Tab::Commands => write_commands(&mut file, app)?,
        Tab::Projects => write_projects(&mut file, app)?,
    }

    Ok(path)
}

fn write_dashboard(f: &mut std::fs::File, app: &App) -> Result<()> {
    let s = &app.cache.summary;
    writeln!(f, "metric,value")?;
    writeln!(f, "total_commands,{}", s.total_commands)?;
    writeln!(f, "total_input,{}", s.total_input)?;
    writeln!(f, "total_output,{}", s.total_output)?;
    writeln!(f, "total_saved,{}", s.total_saved)?;
    writeln!(f, "avg_savings_pct,{:.1}", s.avg_savings_pct)?;
    writeln!(f, "total_time_ms,{}", s.total_time_ms)?;
    writeln!(f, "avg_time_ms,{}", s.avg_time_ms)?;
    writeln!(f, "saved_last_24h,{}", app.cache.saved_last_24h)?;
    Ok(())
}

fn write_daily(f: &mut std::fs::File, app: &App) -> Result<()> {
    writeln!(
        f,
        "date,commands,input_tokens,output_tokens,saved_tokens,savings_pct"
    )?;
    for d in &app.cache.daily {
        writeln!(
            f,
            "{},{},{},{},{},{:.1}",
            d.date, d.commands, d.input_tokens, d.output_tokens, d.saved_tokens, d.savings_pct
        )?;
    }
    Ok(())
}

fn write_weekly(f: &mut std::fs::File, app: &App) -> Result<()> {
    writeln!(f, "week_start,week_end,commands,saved_tokens,savings_pct")?;
    for w in &app.cache.weekly {
        writeln!(
            f,
            "{},{},{},{},{:.1}",
            w.week_start, w.week_end, w.commands, w.saved_tokens, w.savings_pct
        )?;
    }
    Ok(())
}

fn write_monthly(f: &mut std::fs::File, app: &App) -> Result<()> {
    writeln!(f, "month,commands,saved_tokens,savings_pct")?;
    for m in &app.cache.monthly {
        writeln!(
            f,
            "{},{},{},{:.1}",
            m.month, m.commands, m.saved_tokens, m.savings_pct
        )?;
    }
    Ok(())
}

fn write_commands(f: &mut std::fs::File, app: &App) -> Result<()> {
    writeln!(f, "command,count,total_saved,avg_savings_pct")?;
    for c in &app.cache.top_commands {
        writeln!(
            f,
            "\"{}\",{},{},{:.1}",
            c.command.replace('"', "\"\""),
            c.count,
            c.total_saved,
            c.avg_savings_pct
        )?;
    }
    Ok(())
}

fn write_projects(f: &mut std::fs::File, app: &App) -> Result<()> {
    writeln!(f, "project_path,commands,total_saved,savings_pct")?;
    for p in &app.cache.projects {
        writeln!(
            f,
            "\"{}\",{},{},{:.1}",
            p.project_path.replace('"', "\"\""),
            p.commands,
            p.total_saved,
            p.savings_pct
        )?;
    }
    Ok(())
}

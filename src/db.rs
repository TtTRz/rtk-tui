use anyhow::{Context, Result};
use rusqlite::{params, Connection};
use std::collections::HashMap;
use std::path::PathBuf;

/// Read-only handle to RTK's tracking SQLite database.
pub struct Db {
    conn: Connection,
}

// ── Data types ──
// All token counts use i64 to safely represent DB values (including potential negatives).

#[derive(Debug, Default)]
pub struct Summary {
    pub total_commands: i64,
    pub total_input: i64,
    pub total_output: i64,
    pub total_saved: i64,
    pub avg_savings_pct: f64,
    pub total_time_ms: i64,
}

#[derive(Debug)]
pub struct DayStats {
    pub date: String,
    pub commands: i64,
    pub input_tokens: i64,
    pub output_tokens: i64,
    pub saved_tokens: i64,
    pub savings_pct: f64,
}

#[derive(Debug)]
pub struct WeekStats {
    pub week_start: String,
    pub week_end: String,
    pub commands: i64,
    pub saved_tokens: i64,
    pub savings_pct: f64,
}

#[derive(Debug)]
pub struct MonthStats {
    pub month: String,
    pub commands: i64,
    pub saved_tokens: i64,
    pub savings_pct: f64,
}

#[derive(Debug, Default)]
pub struct CommandRecord {
    pub timestamp: String,
    pub rtk_cmd: String,
    pub saved_tokens: i64,
    pub savings_pct: f64,
}

#[derive(Debug)]
pub struct TopCommand {
    pub command: String,
    pub count: i64,
    pub total_saved: i64,
    pub avg_savings_pct: f64,
}

#[derive(Debug)]
pub struct ProjectStats {
    pub project_path: String,
    pub commands: i64,
    pub total_saved: i64,
    pub savings_pct: f64,
}

// ── Helpers ──

fn resolve_db_path(custom: Option<&str>) -> Result<PathBuf> {
    if let Some(p) = custom {
        return Ok(PathBuf::from(p));
    }
    if let Ok(env_path) = std::env::var("RTK_DB_PATH") {
        return Ok(PathBuf::from(env_path));
    }
    let data_dir = dirs::data_local_dir().context("Cannot determine data directory")?;
    Ok(data_dir.join("rtk").join("history.db"))
}

/// Escape GLOB metacharacters in a string so it can be used as a literal prefix.
fn glob_escape(s: &str) -> String {
    s.chars()
        .flat_map(|c| match c {
            '*' | '?' | '[' | ']' => vec!['[', c, ']'],
            c => vec![c],
        })
        .collect()
}

impl Db {
    /// Open the tracking database in read-only mode.
    pub fn open(custom_path: Option<&str>) -> Result<Self> {
        let db_path = resolve_db_path(custom_path)?;
        if !db_path.exists() {
            anyhow::bail!(
                "RTK tracking database not found at: {}\nRun some rtk commands first to generate data.",
                db_path.display()
            );
        }
        let conn = Connection::open_with_flags(
            &db_path,
            rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY | rusqlite::OpenFlags::SQLITE_OPEN_NO_MUTEX,
        )
        .with_context(|| format!("Failed to open database: {}", db_path.display()))?;

        conn.execute_batch("PRAGMA query_only = true;")
            .context("Failed to set read-only pragma")?;

        Ok(Self { conn })
    }

    pub fn get_summary(&self, project: Option<&str>) -> Result<Summary> {
        let parse = |row: &rusqlite::Row| -> rusqlite::Result<Summary> {
            Ok(Summary {
                total_commands: row.get(0)?,
                total_input: row.get(1)?,
                total_output: row.get(2)?,
                total_saved: row.get(3)?,
                avg_savings_pct: row.get(4)?,
                total_time_ms: row.get(5)?,
            })
        };

        let result = if let Some(p) = project {
            let glob = format!("{}{}*", glob_escape(p), std::path::MAIN_SEPARATOR);
            self.conn.query_row(
                "SELECT COUNT(*), COALESCE(SUM(input_tokens),0), COALESCE(SUM(output_tokens),0),
                        COALESCE(SUM(saved_tokens),0), COALESCE(AVG(savings_pct),0),
                        COALESCE(SUM(exec_time_ms),0)
                 FROM commands WHERE project_path = ?1 OR project_path GLOB ?2",
                params![p, glob],
                parse,
            )?
        } else {
            self.conn.query_row(
                "SELECT COUNT(*), COALESCE(SUM(input_tokens),0), COALESCE(SUM(output_tokens),0),
                        COALESCE(SUM(saved_tokens),0), COALESCE(AVG(savings_pct),0),
                        COALESCE(SUM(exec_time_ms),0)
                 FROM commands",
                [],
                parse,
            )?
        };
        Ok(result)
    }

    pub fn get_daily(&self, project: Option<&str>) -> Result<Vec<DayStats>> {
        let parse = |row: &rusqlite::Row| -> rusqlite::Result<DayStats> {
            Ok(DayStats {
                date: row.get(0)?,
                commands: row.get(1)?,
                input_tokens: row.get(2)?,
                output_tokens: row.get(3)?,
                saved_tokens: row.get(4)?,
                savings_pct: row.get(5)?,
            })
        };

        if let Some(p) = project {
            let glob = format!("{}{}*", glob_escape(p), std::path::MAIN_SEPARATOR);
            let mut stmt = self.conn.prepare_cached(
                "SELECT DATE(timestamp) as d, COUNT(*), SUM(input_tokens), SUM(output_tokens),
                        SUM(saved_tokens), AVG(savings_pct)
                 FROM commands WHERE project_path = ?1 OR project_path GLOB ?2
                 GROUP BY d ORDER BY d DESC LIMIT 90",
            )?;
            let rows = stmt.query_map(params![p, glob], parse)?;
            rows.map(|r| r.map_err(Into::into)).collect()
        } else {
            let mut stmt = self.conn.prepare_cached(
                "SELECT DATE(timestamp) as d, COUNT(*), SUM(input_tokens), SUM(output_tokens),
                        SUM(saved_tokens), AVG(savings_pct)
                 FROM commands GROUP BY d ORDER BY d DESC LIMIT 90",
            )?;
            let rows = stmt.query_map([], parse)?;
            rows.map(|r| r.map_err(Into::into)).collect()
        }
    }

    pub fn get_weekly(&self, project: Option<&str>) -> Result<Vec<WeekStats>> {
        let parse = |row: &rusqlite::Row| -> rusqlite::Result<WeekStats> {
            Ok(WeekStats {
                week_start: row.get(0)?,
                week_end: row.get(1)?,
                commands: row.get(2)?,
                saved_tokens: row.get(3)?,
                savings_pct: row.get(4)?,
            })
        };

        if let Some(p) = project {
            let glob = format!("{}{}*", glob_escape(p), std::path::MAIN_SEPARATOR);
            let mut stmt = self.conn.prepare_cached(
                "SELECT DATE(timestamp, 'weekday 0', '-6 days') as ws,
                        DATE(timestamp, 'weekday 0') as we,
                        COUNT(*), SUM(saved_tokens), AVG(savings_pct)
                 FROM commands WHERE project_path = ?1 OR project_path GLOB ?2
                 GROUP BY ws ORDER BY ws DESC LIMIT 52",
            )?;
            let rows = stmt.query_map(params![p, glob], parse)?;
            rows.map(|r| r.map_err(Into::into)).collect()
        } else {
            let mut stmt = self.conn.prepare_cached(
                "SELECT DATE(timestamp, 'weekday 0', '-6 days') as ws,
                        DATE(timestamp, 'weekday 0') as we,
                        COUNT(*), SUM(saved_tokens), AVG(savings_pct)
                 FROM commands GROUP BY ws ORDER BY ws DESC LIMIT 52",
            )?;
            let rows = stmt.query_map([], parse)?;
            rows.map(|r| r.map_err(Into::into)).collect()
        }
    }

    pub fn get_monthly(&self, project: Option<&str>) -> Result<Vec<MonthStats>> {
        let parse = |row: &rusqlite::Row| -> rusqlite::Result<MonthStats> {
            Ok(MonthStats {
                month: row.get(0)?,
                commands: row.get(1)?,
                saved_tokens: row.get(2)?,
                savings_pct: row.get(3)?,
            })
        };

        if let Some(p) = project {
            let glob = format!("{}{}*", glob_escape(p), std::path::MAIN_SEPARATOR);
            let mut stmt = self.conn.prepare_cached(
                "SELECT STRFTIME('%Y-%m', timestamp) as m, COUNT(*),
                        SUM(saved_tokens), AVG(savings_pct)
                 FROM commands WHERE project_path = ?1 OR project_path GLOB ?2
                 GROUP BY m ORDER BY m DESC LIMIT 24",
            )?;
            let rows = stmt.query_map(params![p, glob], parse)?;
            rows.map(|r| r.map_err(Into::into)).collect()
        } else {
            let mut stmt = self.conn.prepare_cached(
                "SELECT STRFTIME('%Y-%m', timestamp) as m, COUNT(*),
                        SUM(saved_tokens), AVG(savings_pct)
                 FROM commands GROUP BY m ORDER BY m DESC LIMIT 24",
            )?;
            let rows = stmt.query_map([], parse)?;
            rows.map(|r| r.map_err(Into::into)).collect()
        }
    }

    pub fn get_recent(&self, limit: usize) -> Result<Vec<CommandRecord>> {
        let mut stmt = self.conn.prepare_cached(
            "SELECT timestamp, rtk_cmd, saved_tokens, savings_pct
             FROM commands ORDER BY timestamp DESC LIMIT ?1",
        )?;
        let rows = stmt.query_map(params![limit as i64], |row| {
            Ok(CommandRecord {
                timestamp: row.get(0)?,
                rtk_cmd: row.get(1)?,
                saved_tokens: row.get(2)?,
                savings_pct: row.get(3)?,
            })
        })?;
        rows.map(|r| r.map_err(Into::into)).collect()
    }

    pub fn get_top_commands(&self, limit: usize) -> Result<Vec<TopCommand>> {
        let mut stmt = self.conn.prepare_cached(
            "SELECT rtk_cmd, COUNT(*) as cnt, SUM(saved_tokens), AVG(savings_pct)
             FROM commands GROUP BY rtk_cmd ORDER BY SUM(saved_tokens) DESC LIMIT ?1",
        )?;
        let rows = stmt.query_map(params![limit as i64], |row| {
            Ok(TopCommand {
                command: row.get(0)?,
                count: row.get(1)?,
                total_saved: row.get(2)?,
                avg_savings_pct: row.get(3)?,
            })
        })?;
        rows.map(|r| r.map_err(Into::into)).collect()
    }

    pub fn get_projects(&self) -> Result<Vec<ProjectStats>> {
        let mut stmt = self.conn.prepare_cached(
            "SELECT project_path, COUNT(*), SUM(saved_tokens), AVG(savings_pct)
             FROM commands WHERE project_path != ''
             GROUP BY project_path ORDER BY SUM(saved_tokens) DESC LIMIT 500",
        )?;
        let rows = stmt.query_map([], |row| {
            Ok(ProjectStats {
                project_path: row.get(0)?,
                commands: row.get(1)?,
                total_saved: row.get(2)?,
                savings_pct: row.get(3)?,
            })
        })?;
        rows.map(|r| r.map_err(Into::into)).collect()
    }

    /// Get daily savings for the last N days (for sparkline).
    /// Single query with GROUP BY, filling in zeros for missing days.
    pub fn get_daily_sparkline(&self, days: usize) -> Result<Vec<u64>> {
        let offset = format!("-{} days", days.saturating_sub(1));
        let mut stmt = self.conn.prepare_cached(
            "SELECT DATE(timestamp) as d, COALESCE(SUM(saved_tokens), 0)
             FROM commands
             WHERE DATE(timestamp) >= DATE('now', ?1)
             GROUP BY d ORDER BY d ASC",
        )?;

        let map: HashMap<String, i64> = stmt
            .query_map(params![offset], |row| {
                Ok((row.get::<_, String>(0)?, row.get::<_, i64>(1)?))
            })?
            .filter_map(|r| r.ok())
            .collect();

        let today = chrono::Local::now().date_naive();
        let values: Vec<u64> = (0..days)
            .map(|i| {
                let date = (today - chrono::Duration::days((days - 1 - i) as i64))
                    .format("%Y-%m-%d")
                    .to_string();
                map.get(&date).copied().unwrap_or(0).max(0) as u64
            })
            .collect();

        Ok(values)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_db() -> (tempfile::TempDir, Db) {
        let dir = tempfile::TempDir::new().unwrap();
        let db_path = dir.path().join("test.db");

        let conn = Connection::open(&db_path).unwrap();
        conn.execute_batch(
            "CREATE TABLE commands (
                id INTEGER PRIMARY KEY,
                timestamp TEXT NOT NULL,
                original_cmd TEXT NOT NULL,
                rtk_cmd TEXT NOT NULL,
                input_tokens INTEGER NOT NULL,
                output_tokens INTEGER NOT NULL,
                saved_tokens INTEGER NOT NULL,
                savings_pct REAL NOT NULL,
                exec_time_ms INTEGER DEFAULT 0,
                project_path TEXT DEFAULT ''
            );
            INSERT INTO commands VALUES (1, '2026-04-14T10:00:00Z', 'git status', 'rtk git status', 1000, 200, 800, 80.0, 5, '/home/user/project');
            INSERT INTO commands VALUES (2, '2026-04-14T11:00:00Z', 'cargo test', 'rtk cargo test', 5000, 500, 4500, 90.0, 10, '/home/user/project');
            INSERT INTO commands VALUES (3, '2026-04-13T09:00:00Z', 'git log', 'rtk git log', 2000, 400, 1600, 80.0, 3, '/home/user/other');",
        )
        .unwrap();
        drop(conn);

        let db = Db::open(Some(db_path.to_str().unwrap())).unwrap();
        (dir, db)
    }

    #[test]
    fn test_summary_global() {
        let (_dir, db) = create_test_db();
        let s = db.get_summary(None).unwrap();
        assert_eq!(s.total_commands, 3);
        assert_eq!(s.total_saved, 6900);
    }

    #[test]
    fn test_summary_project_filtered() {
        let (_dir, db) = create_test_db();
        let s = db.get_summary(Some("/home/user/project")).unwrap();
        assert_eq!(s.total_commands, 2);
        assert_eq!(s.total_saved, 5300);
    }

    #[test]
    fn test_recent() {
        let (_dir, db) = create_test_db();
        let recent = db.get_recent(2).unwrap();
        assert_eq!(recent.len(), 2);
        assert_eq!(recent[0].rtk_cmd, "rtk cargo test");
    }

    #[test]
    fn test_top_commands() {
        let (_dir, db) = create_test_db();
        let top = db.get_top_commands(10).unwrap();
        assert!(!top.is_empty());
        assert_eq!(top[0].command, "rtk cargo test");
    }

    #[test]
    fn test_projects() {
        let (_dir, db) = create_test_db();
        let projects = db.get_projects().unwrap();
        assert_eq!(projects.len(), 2);
    }

    #[test]
    fn test_daily() {
        let (_dir, db) = create_test_db();
        let daily = db.get_daily(None).unwrap();
        assert!(!daily.is_empty());
    }

    #[test]
    fn test_weekly() {
        let (_dir, db) = create_test_db();
        let weekly = db.get_weekly(None).unwrap();
        assert!(!weekly.is_empty());
    }

    #[test]
    fn test_monthly() {
        let (_dir, db) = create_test_db();
        let monthly = db.get_monthly(None).unwrap();
        assert!(!monthly.is_empty());
    }

    #[test]
    fn test_sparkline_returns_correct_length() {
        let (_dir, db) = create_test_db();
        let sparkline = db.get_daily_sparkline(30).unwrap();
        assert_eq!(sparkline.len(), 30);
    }

    #[test]
    fn test_sparkline_fills_zeros_for_missing_days() {
        let (_dir, db) = create_test_db();
        let sparkline = db.get_daily_sparkline(7).unwrap();
        assert_eq!(sparkline.len(), 7);
        // Most days should be 0 (only 2 days have data)
        let zero_count = sparkline.iter().filter(|&&v| v == 0).count();
        assert!(
            zero_count >= 5,
            "Expected at least 5 zero days, got {zero_count}"
        );
    }

    #[test]
    fn test_glob_escape() {
        assert_eq!(glob_escape("normal/path"), "normal/path");
        assert_eq!(glob_escape("path[with]brackets"), "path[[]with[]]brackets");
        assert_eq!(glob_escape("has*star"), "has[*]star");
        assert_eq!(glob_escape("has?question"), "has[?]question");
    }

    #[test]
    fn test_db_not_found() {
        let result = Db::open(Some("/nonexistent/path/db.sqlite"));
        assert!(result.is_err());
    }
}

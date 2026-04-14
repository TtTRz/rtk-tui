use anyhow::Result;
use ratatui::{DefaultTerminal, Frame};
use std::sync::mpsc;
use std::time::Duration;

use crate::db::{self, Db};
use crate::event::{spawn_event_threads, AppEvent};
use crate::ui;

/// Which tab is currently active.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Tab {
    Dashboard,
    History,
    Commands,
    Projects,
}

impl Tab {
    pub const ALL: [Tab; 4] = [Tab::Dashboard, Tab::History, Tab::Commands, Tab::Projects];

    pub fn title(self) -> &'static str {
        match self {
            Tab::Dashboard => "Dashboard",
            Tab::History => "History",
            Tab::Commands => "Commands",
            Tab::Projects => "Projects",
        }
    }

    pub fn index(self) -> usize {
        Tab::ALL
            .iter()
            .position(|&t| t == self)
            .expect("Tab not found in ALL")
    }

    fn next(self) -> Tab {
        Tab::ALL[(self.index() + 1) % Tab::ALL.len()]
    }
}

/// History sub-view granularity.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum HistoryView {
    Daily,
    Weekly,
    Monthly,
}

/// Cached data snapshot — refreshed only on DB change or tick.
#[derive(Default)]
pub struct DataCache {
    pub summary: db::Summary,
    pub sparkline: Vec<u64>,
    pub recent: Vec<db::CommandRecord>,
    pub daily: Vec<db::DayStats>,
    pub weekly: Vec<db::WeekStats>,
    pub monthly: Vec<db::MonthStats>,
    pub top_commands: Vec<db::TopCommand>,
    pub projects: Vec<db::ProjectStats>,
}

pub struct App {
    pub db: Db,
    pub cache: DataCache,
    pub tab: Tab,
    pub history_view: HistoryView,
    pub scroll_offset: usize,
    pub last_error: Option<String>,
    pub should_quit: bool,
    needs_redraw: bool,
    refresh_interval: Duration,
}

impl App {
    pub fn new(db: Db, refresh_secs: u64) -> Self {
        let mut app = Self {
            db,
            cache: DataCache::default(),
            tab: Tab::Dashboard,
            history_view: HistoryView::Daily,
            scroll_offset: 0,
            last_error: None,
            should_quit: false,
            needs_redraw: true,
            refresh_interval: Duration::from_secs(refresh_secs),
        };
        app.refresh_cache();
        app
    }

    pub fn run(&mut self, terminal: &mut DefaultTerminal) -> Result<()> {
        let (tx, rx) = mpsc::channel();
        spawn_event_threads(tx, self.refresh_interval);

        loop {
            if self.needs_redraw {
                terminal.draw(|frame| self.draw(frame))?;
                self.needs_redraw = false;
            }

            // Block until next event (key press, tick, or resize)
            match rx.recv()? {
                AppEvent::Key(code) => self.handle_key(code),
                AppEvent::Tick => self.on_tick(),
                AppEvent::Resize => self.needs_redraw = true,
            }

            if self.should_quit {
                return Ok(());
            }
        }
    }

    fn draw(&self, frame: &mut Frame) {
        ui::render(frame, self);
    }

    fn handle_key(&mut self, code: crossterm::event::KeyCode) {
        use crossterm::event::KeyCode;

        match code {
            KeyCode::Char('q') | KeyCode::Esc => self.should_quit = true,
            KeyCode::Tab => {
                self.tab = self.tab.next();
                self.scroll_offset = 0;
                self.needs_redraw = true;
            }
            KeyCode::Char('1') => self.switch_tab(Tab::Dashboard),
            KeyCode::Char('2') => self.switch_tab(Tab::History),
            KeyCode::Char('3') => self.switch_tab(Tab::Commands),
            KeyCode::Char('4') => self.switch_tab(Tab::Projects),
            // History sub-view
            KeyCode::Char('d') if self.tab == Tab::History => {
                self.history_view = HistoryView::Daily;
                self.scroll_offset = 0;
                self.needs_redraw = true;
            }
            KeyCode::Char('w') if self.tab == Tab::History => {
                self.history_view = HistoryView::Weekly;
                self.scroll_offset = 0;
                self.needs_redraw = true;
            }
            KeyCode::Char('m') if self.tab == Tab::History => {
                self.history_view = HistoryView::Monthly;
                self.scroll_offset = 0;
                self.needs_redraw = true;
            }
            // Scroll (clamped to data length)
            KeyCode::Down | KeyCode::Char('j') => {
                let max = self.max_scroll();
                self.scroll_offset = self.scroll_offset.saturating_add(1).min(max);
                self.needs_redraw = true;
            }
            KeyCode::Up | KeyCode::Char('k') => {
                self.scroll_offset = self.scroll_offset.saturating_sub(1);
                self.needs_redraw = true;
            }
            // Force refresh
            KeyCode::Char('r') => {
                self.refresh_cache();
                self.needs_redraw = true;
            }
            _ => {}
        }
    }

    fn switch_tab(&mut self, tab: Tab) {
        if self.tab != tab {
            self.tab = tab;
            self.scroll_offset = 0;
            self.needs_redraw = true;
        }
    }

    fn on_tick(&mut self) {
        self.refresh_cache();
        self.needs_redraw = true;
    }

    /// Maximum valid scroll offset for the current tab/view.
    fn max_scroll(&self) -> usize {
        let len = match self.tab {
            Tab::Dashboard => 0,
            Tab::History => match self.history_view {
                HistoryView::Daily => self.cache.daily.len(),
                HistoryView::Weekly => self.cache.weekly.len(),
                HistoryView::Monthly => self.cache.monthly.len(),
            },
            Tab::Commands => self.cache.top_commands.len(),
            Tab::Projects => self.cache.projects.len(),
        };
        len.saturating_sub(1)
    }

    /// Refresh all cached data from DB, tracking errors.
    fn refresh_cache(&mut self) {
        self.last_error = None;

        macro_rules! fetch {
            ($expr:expr) => {
                $expr.unwrap_or_else(|e| {
                    if self.last_error.is_none() {
                        self.last_error = Some(e.to_string());
                    }
                    Default::default()
                })
            };
        }

        self.cache = DataCache {
            summary: fetch!(self.db.get_summary(None)),
            sparkline: fetch!(self.db.get_daily_sparkline(30)),
            recent: fetch!(self.db.get_recent(10)),
            daily: fetch!(self.db.get_daily(None)),
            weekly: fetch!(self.db.get_weekly(None)),
            monthly: fetch!(self.db.get_monthly(None)),
            top_commands: fetch!(self.db.get_top_commands(50)),
            projects: fetch!(self.db.get_projects()),
        };
    }
}

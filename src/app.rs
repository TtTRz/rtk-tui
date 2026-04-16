use anyhow::Result;
use ratatui::DefaultTerminal;
use std::sync::mpsc;
use std::time::Duration;

use crate::buddy::BuddyState;
use crate::db::{self, Db};
use crate::event::{AppEvent, spawn_event_threads};
use crate::export;
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

impl HistoryView {
    const ALL: [HistoryView; 3] = [
        HistoryView::Daily,
        HistoryView::Weekly,
        HistoryView::Monthly,
    ];

    fn index(self) -> usize {
        match self {
            HistoryView::Daily => 0,
            HistoryView::Weekly => 1,
            HistoryView::Monthly => 2,
        }
    }
}

/// Cached data snapshot — refreshed only on DB change or tick.
#[derive(Default, PartialEq)]
pub struct DataCache {
    pub summary: db::Summary,
    pub saved_last_24h: i64,
    pub sparkline_24h: Vec<u64>,
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
    pub buddy: BuddyState,
    pub tab: Tab,
    pub history_view: HistoryView,
    tab_scroll_offsets: [usize; 4],
    history_scroll_offsets: [usize; 3],
    pub last_error: Option<String>,
    pub should_quit: bool,
    pub show_help: bool,
    pub search_mode: bool,
    pub search_query: String,
    pub export_msg: Option<String>,
    export_msg_ticks: u8,
    needs_redraw: bool,
    refresh_interval_ticks: usize,
    tick_count: usize,
    last_terminal_width: u16,
    chart_cache_width: usize,
    stretched_sparkline_24h: Vec<u64>,
    stretched_sparkline_30d: Vec<u64>,
    chart_cache_dirty: bool,
}

impl App {
    pub fn new(db: Db, refresh_secs: u64, db_path: &str, buddy_species: Option<&str>) -> Self {
        // Animation tick is 500ms, data refresh every refresh_secs
        let refresh_interval_ticks = (refresh_secs * 2).max(1) as usize;
        let mut app = Self {
            db,
            cache: DataCache::default(),
            buddy: BuddyState::new(db_path, buddy_species),
            tab: Tab::Dashboard,
            history_view: HistoryView::Daily,
            tab_scroll_offsets: [0; 4],
            history_scroll_offsets: [0; 3],
            last_error: None,
            should_quit: false,
            show_help: false,
            search_mode: false,
            search_query: String::new(),
            export_msg: None,
            export_msg_ticks: 0,
            needs_redraw: true,
            refresh_interval_ticks,
            tick_count: 0,
            last_terminal_width: 0,
            chart_cache_width: 0,
            stretched_sparkline_24h: Vec::new(),
            stretched_sparkline_30d: Vec::new(),
            chart_cache_dirty: true,
        };
        app.refresh_cache();
        app
    }

    pub fn run(&mut self, terminal: &mut DefaultTerminal) -> Result<()> {
        let (tx, rx) = mpsc::channel();
        // Animation tick at 500ms (matching Claude Code)
        spawn_event_threads(tx, Duration::from_millis(500));

        loop {
            if self.needs_redraw {
                let term_size = terminal.size()?;
                self.last_terminal_width = term_size.width;
                ui::dashboard::update_buddy_max_x(self, term_size.width);
                self.prepare_chart_cache(term_size.width);
                terminal.draw(|frame| ui::render(frame, self))?;
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

    pub fn scroll_offset(&self) -> usize {
        match self.tab {
            Tab::History => self.history_scroll_offsets[self.history_view.index()],
            _ => self.tab_scroll_offsets[self.tab.index()],
        }
    }

    pub fn chart_cache_width(&self) -> usize {
        self.chart_cache_width
    }

    pub fn stretched_sparkline_24h(&self) -> &[u64] {
        &self.stretched_sparkline_24h
    }

    pub fn stretched_sparkline_30d(&self) -> &[u64] {
        &self.stretched_sparkline_30d
    }

    fn handle_key(&mut self, code: crossterm::event::KeyCode) {
        use crossterm::event::KeyCode;

        // Help popup: any key dismisses
        if self.show_help {
            self.show_help = false;
            self.needs_redraw = true;
            return;
        }

        // Search mode: capture text input
        if self.search_mode {
            match code {
                KeyCode::Esc | KeyCode::Enter => {
                    self.search_mode = false;
                    if code == KeyCode::Esc {
                        self.search_query.clear();
                    }
                    self.clamp_current_scroll_offset();
                    self.needs_redraw = true;
                }
                KeyCode::Backspace => {
                    self.search_query.pop();
                    self.clamp_current_scroll_offset();
                    self.needs_redraw = true;
                }
                KeyCode::Char(c) => {
                    self.search_query.push(c);
                    self.clamp_current_scroll_offset();
                    self.needs_redraw = true;
                }
                _ => {}
            }
            return;
        }

        match code {
            KeyCode::Char('q') | KeyCode::Esc => self.should_quit = true,
            KeyCode::Char('?') => {
                self.show_help = true;
                self.needs_redraw = true;
            }
            KeyCode::Char('/') => {
                self.search_mode = true;
                self.search_query.clear();
                self.clamp_current_scroll_offset();
                self.needs_redraw = true;
            }
            KeyCode::Char('e') => {
                self.handle_export();
                self.needs_redraw = true;
            }
            KeyCode::Tab => self.switch_tab(self.tab.next()),
            KeyCode::Char('1') => self.switch_tab(Tab::Dashboard),
            KeyCode::Char('2') => self.switch_tab(Tab::History),
            KeyCode::Char('3') => self.switch_tab(Tab::Commands),
            KeyCode::Char('4') => self.switch_tab(Tab::Projects),
            // History sub-view
            KeyCode::Char('d') if self.tab == Tab::History => {
                self.switch_history_view(HistoryView::Daily)
            }
            KeyCode::Char('w') if self.tab == Tab::History => {
                self.switch_history_view(HistoryView::Weekly)
            }
            KeyCode::Char('m') if self.tab == Tab::History => {
                self.switch_history_view(HistoryView::Monthly)
            }
            // Scroll (clamped to data length)
            KeyCode::Down | KeyCode::Char('j') => {
                let max = self.max_scroll();
                self.set_current_scroll_offset(self.scroll_offset().saturating_add(1).min(max));
                self.needs_redraw = true;
            }
            KeyCode::Up | KeyCode::Char('k') => {
                self.set_current_scroll_offset(self.scroll_offset().saturating_sub(1));
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
            self.clamp_current_scroll_offset();
            self.needs_redraw = true;
        }
    }

    fn switch_history_view(&mut self, view: HistoryView) {
        if self.history_view != view {
            self.history_view = view;
            self.clamp_current_scroll_offset();
            self.needs_redraw = true;
        }
    }

    fn on_tick(&mut self) {
        self.tick_count += 1;

        let mut needs_redraw = false;

        // Refresh data from DB at the user-configured interval.
        if self.tick_count.is_multiple_of(self.refresh_interval_ticks) {
            needs_redraw |= self.refresh_cache();
        }

        // Buddy animation runs every tick (500ms).
        let buddy_changed = self.buddy.tick(&self.cache);
        let buddy_visible = self.tab == Tab::Dashboard
            && self.last_terminal_width >= ui::layout::BUDDY_MIN_WIDTH
            && !self.show_help;
        needs_redraw |= buddy_changed && buddy_visible;

        // Decay export message (counts in data-refresh ticks).
        if self.tick_count.is_multiple_of(self.refresh_interval_ticks) && self.export_msg_ticks > 0
        {
            self.export_msg_ticks -= 1;
            if self.export_msg_ticks == 0 {
                self.export_msg = None;
            }
            needs_redraw = true;
        }

        self.needs_redraw |= needs_redraw;
    }

    /// Maximum valid scroll offset for the current tab/view.
    fn max_scroll(&self) -> usize {
        self.current_item_count().saturating_sub(1)
    }

    fn current_item_count(&self) -> usize {
        match self.tab {
            Tab::Dashboard => 0,
            Tab::History => match self.history_view {
                HistoryView::Daily => self.cache.daily.len(),
                HistoryView::Weekly => self.cache.weekly.len(),
                HistoryView::Monthly => self.cache.monthly.len(),
            },
            Tab::Commands => self.filtered_commands_len(),
            Tab::Projects => self.filtered_projects_len(),
        }
    }

    fn filtered_commands_len(&self) -> usize {
        let query = self.search_query.to_lowercase();
        self.cache
            .top_commands
            .iter()
            .filter(|c| query.is_empty() || c.command.to_lowercase().contains(&query))
            .count()
    }

    fn filtered_projects_len(&self) -> usize {
        let query = self.search_query.to_lowercase();
        self.cache
            .projects
            .iter()
            .filter(|p| query.is_empty() || p.project_path.to_lowercase().contains(&query))
            .count()
    }

    fn set_current_scroll_offset(&mut self, offset: usize) {
        match self.tab {
            Tab::History => self.history_scroll_offsets[self.history_view.index()] = offset,
            _ => self.tab_scroll_offsets[self.tab.index()] = offset,
        }
    }

    fn clamp_current_scroll_offset(&mut self) {
        self.set_current_scroll_offset(self.scroll_offset().min(self.max_scroll()));
    }

    fn clamp_all_scroll_offsets(&mut self) {
        self.tab_scroll_offsets[Tab::Dashboard.index()] = 0;
        self.tab_scroll_offsets[Tab::History.index()] = 0;
        self.tab_scroll_offsets[Tab::Commands.index()] = self.tab_scroll_offsets
            [Tab::Commands.index()]
        .min(self.filtered_commands_len().saturating_sub(1));
        self.tab_scroll_offsets[Tab::Projects.index()] = self.tab_scroll_offsets
            [Tab::Projects.index()]
        .min(self.filtered_projects_len().saturating_sub(1));

        for view in HistoryView::ALL {
            let idx = view.index();
            let len = match view {
                HistoryView::Daily => self.cache.daily.len(),
                HistoryView::Weekly => self.cache.weekly.len(),
                HistoryView::Monthly => self.cache.monthly.len(),
            };
            self.history_scroll_offsets[idx] =
                self.history_scroll_offsets[idx].min(len.saturating_sub(1));
        }
    }

    fn prepare_chart_cache(&mut self, area_width: u16) {
        let target_width = area_width.saturating_sub(2) as usize;
        if !self.chart_cache_dirty && self.chart_cache_width == target_width {
            return;
        }

        self.stretched_sparkline_24h =
            ui::dashboard::stretch_data(&self.cache.sparkline_24h, target_width);
        self.stretched_sparkline_30d =
            ui::dashboard::stretch_data(&self.cache.sparkline, target_width);
        self.chart_cache_width = target_width;
        self.chart_cache_dirty = false;
    }

    fn handle_export(&mut self) {
        match export::export_csv(self) {
            Ok(path) => {
                self.export_msg = Some(format!("Exported to {}", path.display()));
                self.export_msg_ticks = 3; // show for ~3 ticks
            }
            Err(e) => {
                self.last_error = Some(format!("Export failed: {e}"));
            }
        }
    }

    /// Refresh all cached data from DB, tracking errors.
    fn refresh_cache(&mut self) -> bool {
        let previous_error = self.last_error.clone();
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

        let new_cache = DataCache {
            summary: fetch!(self.db.get_summary(None)),
            saved_last_24h: fetch!(self.db.get_saved_last_24h()),
            sparkline_24h: fetch!(self.db.get_hourly_sparkline(24)),
            sparkline: fetch!(self.db.get_daily_sparkline(30)),
            recent: fetch!(self.db.get_recent(10)),
            daily: fetch!(self.db.get_daily(None)),
            weekly: fetch!(self.db.get_weekly(None)),
            monthly: fetch!(self.db.get_monthly(None)),
            top_commands: fetch!(self.db.get_top_commands(50)),
            projects: fetch!(self.db.get_projects()),
        };

        let cache_changed = self.cache != new_cache;
        let error_changed = self.last_error != previous_error;

        self.cache = new_cache;
        self.clamp_all_scroll_offsets();
        if cache_changed {
            self.chart_cache_dirty = true;
        }

        cache_changed || error_changed
    }
}

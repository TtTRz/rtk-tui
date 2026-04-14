use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use std::sync::mpsc;
use std::time::Duration;

/// Events sent to the main App loop via channel.
pub enum AppEvent {
    /// A key was pressed.
    Key(KeyCode),
    /// Timer tick — time to refresh data.
    Tick,
    /// Terminal was resized.
    Resize,
}

/// Spawn background threads for input polling and tick generation.
///
/// - **Input thread**: polls crossterm for key presses and resize events.
/// - **Tick thread**: sends a Tick at `refresh_interval` to trigger data refresh.
///
/// Both threads exit when the channel is dropped (App exits).
pub fn spawn_event_threads(tx: mpsc::Sender<AppEvent>, refresh_interval: Duration) {
    // Input thread — polls crossterm events with 50ms granularity
    let tx_input = tx.clone();
    std::thread::spawn(move || {
        loop {
            if event::poll(Duration::from_millis(50)).unwrap_or(false) {
                match event::read() {
                    Ok(Event::Key(key)) if key.kind == KeyEventKind::Press => {
                        if tx_input.send(AppEvent::Key(key.code)).is_err() {
                            return; // channel closed, app exited
                        }
                    }
                    Ok(Event::Resize(_, _)) => {
                        if tx_input.send(AppEvent::Resize).is_err() {
                            return;
                        }
                    }
                    _ => {}
                }
            }
        }
    });

    // Tick thread — fires at refresh_interval
    std::thread::spawn(move || {
        loop {
            std::thread::sleep(refresh_interval);
            if tx.send(AppEvent::Tick).is_err() {
                return; // channel closed, app exited
            }
        }
    });
}

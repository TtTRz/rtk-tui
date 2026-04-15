//! Action state machine, timing, PRNG, and animation sequencing.

use super::fnv1a;
use super::mood::Mood;

// ── Timing constants (500ms per tick) ──

const IDLE_TICKS_MIN: usize = 4; // 2s
const IDLE_TICKS_RANGE: usize = 4; // +0..2s
const WALK_TICKS_MIN: usize = 6; // 3s
const WALK_TICKS_RANGE: usize = 6; // +0..3s
const BOUNCE_TICKS_MIN: usize = 2; // 1s
const BOUNCE_TICKS_RANGE: usize = 2; // +0..1s

const BUBBLE_INTERVAL_MIN: usize = 40; // ~20s
const BUBBLE_INTERVAL_RANGE: usize = 20; // +0..10s
pub const BUBBLE_DURATION: u16 = 16; // ~8s

/// Idle animation sequence. Values: 0/1/2 = frame index, -1 = blink on frame 0.
const IDLE_SEQUENCE: [i8; 24] = [
    0, 0, 0, 0, 0, 1, 1, 0, 0, 0, -1, 0, 0, 0, 0, 0, 0, 2, 2, 0, 0, -1, 0, 0,
];

// ── Action enum ──

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Action {
    Idle,
    WalkLeft,
    WalkRight,
    Bounce,
}

// ── Movement result (returned to BuddyState each tick) ──

pub struct TickResult {
    pub x_pos: usize,
    pub facing_left: bool,
    pub bounce_phase: bool,
    pub bubble_text: Option<String>,
}

// ── AnimState ──

pub struct AnimState {
    pub tick_count: usize,
    // Movement
    pub action: Action,
    pub action_ticks_left: usize,
    pub x_pos: usize,
    pub facing_left: bool,
    pub bounce_phase: bool,
    pub max_x: usize,
    // Bubble
    pub bubble_text: Option<String>,
    pub bubble_ticks_left: u16,
    pub next_bubble_in: usize,
    // PRNG
    rng_state: u32,
}

impl AnimState {
    pub fn new(seed: &str) -> Self {
        Self {
            tick_count: 0,
            action: Action::Idle,
            action_ticks_left: 6, // start with short idle
            x_pos: 4,
            facing_left: false,
            bounce_phase: false,
            max_x: 14,
            bubble_text: None,
            bubble_ticks_left: 0,
            next_bubble_in: BUBBLE_INTERVAL_MIN,
            rng_state: fnv1a(seed),
        }
    }

    /// Advance animation by one tick. Returns movement + bubble state.
    pub fn tick(&mut self, mood: Mood) -> TickResult {
        self.tick_count += 1;

        // Action state machine
        if self.action_ticks_left == 0 {
            self.pick_next_action(mood);
        }
        self.execute_action();
        self.action_ticks_left = self.action_ticks_left.saturating_sub(1);

        // Bubble lifecycle
        self.update_bubble(mood);

        TickResult {
            x_pos: self.x_pos,
            facing_left: self.facing_left,
            bounce_phase: self.bounce_phase,
            bubble_text: self.bubble_text.clone(),
        }
    }

    /// Get (frame_index, is_blink) for the current tick.
    pub fn current_frame_info(&self) -> (usize, bool) {
        match self.action {
            Action::WalkLeft | Action::WalkRight => {
                // Alternate frames 0 and 1 for walk steps
                let f = if self.tick_count.is_multiple_of(2) {
                    0
                } else {
                    1
                };
                (f, false)
            }
            Action::Bounce => (2, false),
            Action::Idle => {
                let seq_val = IDLE_SEQUENCE[self.tick_count % IDLE_SEQUENCE.len()];
                if seq_val < 0 {
                    (0, true)
                } else {
                    (seq_val as usize, false)
                }
            }
        }
    }

    /// Force a specific action (used in tests).
    #[cfg(test)]
    pub fn force_action(&mut self, action: Action, ticks: usize) {
        self.action = action;
        self.action_ticks_left = ticks;
    }

    // ── Private ──

    fn pick_next_action(&mut self, mood: Mood) {
        if mood == Mood::Sleepy {
            self.action = Action::Idle;
            self.action_ticks_left = IDLE_TICKS_MIN + (self.next_rng() as usize % IDLE_TICKS_RANGE);
            return;
        }

        let roll = self.next_rng() % 100;
        if roll < 35 {
            self.action = Action::WalkRight;
            self.facing_left = false;
            self.action_ticks_left = WALK_TICKS_MIN + (self.next_rng() as usize % WALK_TICKS_RANGE);
        } else if roll < 70 {
            self.action = Action::WalkLeft;
            self.facing_left = true;
            self.action_ticks_left = WALK_TICKS_MIN + (self.next_rng() as usize % WALK_TICKS_RANGE);
        } else if roll < 85 {
            self.action = Action::Bounce;
            self.action_ticks_left =
                BOUNCE_TICKS_MIN + (self.next_rng() as usize % BOUNCE_TICKS_RANGE);
        } else {
            self.action = Action::Idle;
            self.action_ticks_left = IDLE_TICKS_MIN + (self.next_rng() as usize % IDLE_TICKS_RANGE);
        }
    }

    fn execute_action(&mut self) {
        match self.action {
            Action::Idle => {}
            Action::WalkRight => {
                if self.x_pos < self.max_x {
                    self.x_pos += 1;
                } else {
                    self.action = Action::WalkLeft;
                    self.facing_left = true;
                }
            }
            Action::WalkLeft => {
                if self.x_pos > 0 {
                    self.x_pos -= 1;
                } else {
                    self.action = Action::WalkRight;
                    self.facing_left = false;
                }
            }
            Action::Bounce => {
                if self.tick_count.is_multiple_of(2) {
                    self.bounce_phase = !self.bounce_phase;
                }
            }
        }
    }

    fn update_bubble(&mut self, mood: Mood) {
        if self.bubble_ticks_left > 0 {
            self.bubble_ticks_left -= 1;
            if self.bubble_ticks_left == 0 {
                self.bubble_text = None;
            }
        } else {
            self.next_bubble_in = self.next_bubble_in.saturating_sub(1);
            if self.next_bubble_in == 0 {
                let messages = mood.messages();
                let idx = self.next_rng() as usize % messages.len();
                self.bubble_text = Some(messages[idx].to_string());
                self.bubble_ticks_left = BUBBLE_DURATION;
                self.next_bubble_in =
                    BUBBLE_INTERVAL_MIN + (self.next_rng() as usize % BUBBLE_INTERVAL_RANGE);
            }
        }
    }

    fn next_rng(&mut self) -> u32 {
        self.rng_state ^= self.rng_state << 13;
        self.rng_state ^= self.rng_state >> 17;
        self.rng_state ^= self.rng_state << 5;
        self.rng_state
    }
}

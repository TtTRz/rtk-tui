//! Buddy companion system — a Tamagotchi-style ASCII pet for the dashboard.
//!
//! Architecture:
//! - `species.rs`   — Species enum + ASCII sprite frames
//! - `mood.rs`      — Mood enum + eye chars + message pools
//! - `animation.rs` — Action state machine + timing constants + PRNG
//! - `render.rs`    — Buddy-specific rendering (sprite + bubble positioning)
//! - `mod.rs`       — BuddyState (public API combining all the above)

mod animation;
mod mood;
mod render;
mod species;

use crate::app::DataCache;

pub use mood::Mood;
pub use render::render_buddy;
pub use species::Species;

/// FNV-1a hash for deterministic species selection.
fn fnv1a(s: &str) -> u32 {
    let mut hash: u32 = 2_166_136_261;
    for byte in s.bytes() {
        hash ^= byte as u32;
        hash = hash.wrapping_mul(16_777_619);
    }
    hash
}

/// Complete buddy state, updated every tick.
pub struct BuddyState {
    pub species: Species,
    pub mood: Mood,
    pub bubble_text: Option<String>,
    pub x_pos: usize,
    pub facing_left: bool,
    pub bounce_phase: bool,
    anim: animation::AnimState,
}

impl BuddyState {
    pub fn new(db_path: &str, species_name: Option<&str>) -> Self {
        let species = species_name
            .and_then(Species::from_name)
            .unwrap_or_else(|| Species::from_hash(db_path));
        Self {
            species,
            mood: Mood::Happy,
            bubble_text: Some("Hello! I'm your buddy!".to_string()),
            x_pos: 4,
            facing_left: false,
            bounce_phase: false,
            anim: animation::AnimState::new(db_path),
        }
    }

    /// Set the max x position based on panel width (sprite is ~12 chars wide).
    pub fn set_max_x(&mut self, panel_inner_width: usize) {
        self.anim.max_x = panel_inner_width.saturating_sub(12);
    }

    /// Advance all state. Called every tick (500ms).
    pub fn tick(&mut self, cache: &DataCache) {
        self.mood = Mood::from_stats(cache);
        let movement = self.anim.tick(self.mood);

        // Apply movement results
        self.x_pos = movement.x_pos;
        self.facing_left = movement.facing_left;
        self.bounce_phase = movement.bounce_phase;
        self.bubble_text = movement.bubble_text;
    }

    /// Get current sprite frame lines with eyes substituted.
    pub fn current_frame(&self) -> Vec<String> {
        let (frame_idx, blink) = self.anim.current_frame_info();

        let frames = self.species.frames();
        let frame = &frames[frame_idx.min(2)];

        let eye = if blink || self.mood == Mood::Sleepy {
            "-"
        } else {
            self.mood.eye_char()
        };

        frame.iter().map(|line| line.replace("{E}", eye)).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_species_from_hash_deterministic() {
        let s1 = Species::from_hash("/some/path/db.sqlite");
        let s2 = Species::from_hash("/some/path/db.sqlite");
        assert_eq!(s1, s2);
    }

    #[test]
    fn test_species_from_name() {
        assert_eq!(Species::from_name("llama"), Some(Species::Llama));
        assert_eq!(Species::from_name("LLAMA"), Some(Species::Llama));
        assert_eq!(Species::from_name("cat"), Some(Species::Cat));
        assert_eq!(Species::from_name("nope"), None);
    }

    #[test]
    fn test_mood_from_stats_sleepy() {
        let cache = DataCache::default();
        assert_eq!(Mood::from_stats(&cache), Mood::Sleepy);
    }

    #[test]
    fn test_mood_eye_chars() {
        assert_eq!(Mood::Ecstatic.eye_char(), "✦");
        assert_eq!(Mood::Sleepy.eye_char(), "-");
    }

    #[test]
    fn test_mood_messages_nonempty() {
        for mood in [
            Mood::Ecstatic,
            Mood::Happy,
            Mood::Content,
            Mood::Sleepy,
            Mood::Worried,
        ] {
            assert!(!mood.messages().is_empty());
        }
    }

    #[test]
    fn test_buddy_current_frame() {
        let buddy = BuddyState::new("/test/db", None);
        let frame = buddy.current_frame();
        assert_eq!(frame.len(), 6);
        for line in &frame {
            assert!(!line.contains("{E}"));
        }
    }

    #[test]
    fn test_buddy_tick_advances() {
        let mut buddy = BuddyState::new("/test/db", None);
        let cache = DataCache::default();
        buddy.tick(&cache);
        assert_eq!(buddy.anim.tick_count, 1);
    }

    #[test]
    fn test_buddy_walks() {
        let mut buddy = BuddyState::new("/test/db", None);
        buddy.anim.force_action(animation::Action::WalkRight, 100);
        buddy.x_pos = 0;
        buddy.anim.x_pos = 0;
        buddy.anim.max_x = 10;
        let cache = DataCache::default();
        for _ in 0..2 {
            buddy.tick(&cache);
        }
        assert!(buddy.x_pos > 0, "Buddy should have moved right");
    }

    #[test]
    fn test_buddy_bounces_at_wall() {
        let mut buddy = BuddyState::new("/test/db", None);
        buddy.anim.force_action(animation::Action::WalkRight, 100);
        buddy.anim.x_pos = 10;
        buddy.anim.max_x = 10;
        let cache = DataCache::default();
        for _ in 0..2 {
            buddy.tick(&cache);
        }
        assert_eq!(buddy.anim.action, animation::Action::WalkLeft);
    }

    #[test]
    fn test_buddy_bubble_appears() {
        let mut buddy = BuddyState::new("/test/db", None);
        buddy.anim.bubble_ticks_left = 0;
        buddy.bubble_text = None;
        buddy.anim.next_bubble_in = 1;
        let cache = DataCache::default();
        buddy.tick(&cache);
        assert!(buddy.bubble_text.is_some());
    }

    #[test]
    fn test_fnv1a_deterministic() {
        assert_eq!(fnv1a("hello"), fnv1a("hello"));
        assert_ne!(fnv1a("hello"), fnv1a("world"));
    }
}

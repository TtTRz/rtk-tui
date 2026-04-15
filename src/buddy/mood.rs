//! Mood system — emotion state driven by token savings data.

use crate::app::DataCache;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Mood {
    Ecstatic,
    Happy,
    Content,
    Sleepy,
    Worried,
}

impl Mood {
    /// Determine mood from current data cache.
    pub fn from_stats(cache: &DataCache) -> Self {
        let efficiency = cache.summary.avg_savings_pct;
        let saved_24h = cache.saved_last_24h;

        if saved_24h == 0 {
            Self::Sleepy
        } else if saved_24h > 50_000 {
            Self::Ecstatic
        } else if efficiency >= 80.0 {
            Self::Happy
        } else if efficiency >= 50.0 {
            Self::Content
        } else {
            Self::Worried
        }
    }

    /// Eye character for this mood.
    pub fn eye_char(self) -> &'static str {
        match self {
            Self::Ecstatic => "✦",
            Self::Happy => "·",
            Self::Content => "°",
            Self::Sleepy => "-",
            Self::Worried => "×",
        }
    }

    /// Pool of speech bubble messages for this mood.
    pub fn messages(self) -> &'static [&'static str] {
        match self {
            Self::Ecstatic => &[
                "Incredible savings today!",
                "You're on fire!",
                "Token efficiency master!",
                "RTK is doing amazing work!",
                "50K+ saved? Wow!",
                "Best day ever!",
            ],
            Self::Happy => &[
                "Keep it up!",
                "Great efficiency!",
                "Tokens well saved.",
                "RTK is happy!",
                "Smooth sailing~",
                "Nice work today!",
            ],
            Self::Content => &[
                "Doing good...",
                "Steady progress.",
                "Not bad at all.",
                "Hmm, decent.",
                "Room to grow~",
                "Chipping away...",
            ],
            Self::Sleepy => &[
                "zzZ",
                "...",
                "*yawn*",
                "So quiet today...",
                "Waiting for action~",
                "Wake me up later.",
            ],
            Self::Worried => &[
                "Let's optimize more...",
                "We can do better!",
                "Try rtk for more cmds?",
                "Savings are low...",
                "Need more filters!",
                "Don't give up!",
            ],
        }
    }
}

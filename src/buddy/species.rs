//! Species definitions and ASCII sprite frames.

use super::fnv1a;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Species {
    Llama,
    Cat,
    Duck,
    Blob,
    Robot,
    Penguin,
    Ghost,
}

impl Species {
    const ALL: [Species; 7] = [
        Species::Llama,
        Species::Cat,
        Species::Duck,
        Species::Blob,
        Species::Robot,
        Species::Penguin,
        Species::Ghost,
    ];

    /// Deterministic species selection from a string hash.
    pub fn from_hash(seed: &str) -> Self {
        let hash = fnv1a(seed);
        Self::ALL[hash as usize % Self::ALL.len()]
    }

    /// Parse species name (case-insensitive).
    pub fn from_name(name: &str) -> Option<Self> {
        match name.to_lowercase().as_str() {
            "llama" => Some(Self::Llama),
            "cat" => Some(Self::Cat),
            "duck" => Some(Self::Duck),
            "blob" => Some(Self::Blob),
            "robot" => Some(Self::Robot),
            "penguin" => Some(Self::Penguin),
            "ghost" => Some(Self::Ghost),
            _ => None,
        }
    }

    /// 3 animation frames, each 6 lines. `{E}` is the eye placeholder.
    pub fn frames(&self) -> [[&'static str; 6]; 3] {
        match self {
            Self::Llama => LLAMA_FRAMES,
            Self::Cat => CAT_FRAMES,
            Self::Duck => DUCK_FRAMES,
            Self::Blob => BLOB_FRAMES,
            Self::Robot => ROBOT_FRAMES,
            Self::Penguin => PENGUIN_FRAMES,
            Self::Ghost => GHOST_FRAMES,
        }
    }
}

// ── Sprite data ──

// "    /)  /)    ",
// "  ( ✦   ✦ )  ",
// "  ((  ᵕ  ))  ",
// " __| --- |__  ",
// "|  |     |  | ",
const LLAMA_FRAMES: [[&str; 6]; 3] = [
    [
        "              ",
        "    /)  /)    ",
        "  ( {E}   {E} )  ",
        "  ((  ᵕ  ))  ",
        " __| --- |__ ",
        "|  |     |  |",
    ],
    [
        "              ",
        "    /)  /)    ",
        "  ( {E}   {E} )  ",
        "  ((  ᵕ  )) ",
        "  __| --- |__ ",
        " |  |     |  |",
    ],
    [
        "              ",
        "    /)  /)    ",
        "  ( {E}   {E} )~ ",
        "  ((  ᵕ  ))  ",
        " __| --- |__ ",
        "|  |     |  |",
    ],
];

const CAT_FRAMES: [[&str; 6]; 3] = [
    [
        "              ",
        "            ",
        "   /\\_/\\    ",
        "  ( {E}  {E} )  ",
        "  (  ω  )   ",
        "  (\")_(\")   ",
    ],
    [
        "              ",
        "            ",
        "   /\\_/\\    ",
        "  ( {E}  {E} )  ",
        "  (  ω  )   ",
        " (\") _(\")   ",
    ],
    [
        "              ",
        "            ",
        "   /\\_/\\    ",
        "  ( {E}  {E} )~ ",
        "  (  ω  )   ",
        "  (\")_(\")   ",
    ],
];

const DUCK_FRAMES: [[&str; 6]; 3] = [
    [
        "              ",
        "            ",
        "    __      ",
        "  <({E} )___ ",
        "   (  ._>   ",
        "    `--´    ",
    ],
    [
        "              ",
        "            ",
        "    __      ",
        "  <({E} )___ ",
        "   (  ._>   ",
        "   ~`--´    ",
    ],
    [
        "              ",
        "            ",
        "    __      ",
        "  <({E} )___ ",
        "   ( .->    ",
        "    `--´    ",
    ],
];

const BLOB_FRAMES: [[&str; 6]; 3] = [
    [
        "              ",
        "            ",
        "   .----.   ",
        "  ( {E}  {E} )  ",
        "  (      )  ",
        "   `----´   ",
    ],
    [
        "              ",
        "            ",
        "   .----.   ",
        "  ( {E}  {E} )  ",
        "  ( .  . )  ",
        "   `----´   ",
    ],
    [
        "              ",
        "            ",
        "    .---.   ",
        "  ( {E}  {E} )  ",
        "  (      )  ",
        "   `----´   ",
    ],
];

const ROBOT_FRAMES: [[&str; 6]; 3] = [
    [
        "              ",
        "            ",
        "   .[||].   ",
        "  [ {E}  {E} ]  ",
        "  [ ==== ]  ",
        "   `----´   ",
    ],
    [
        "              ",
        "            ",
        "   .[||].   ",
        "  [ {E}  {E} ]  ",
        "  [ ==== ]  ",
        "  ~`----´   ",
    ],
    [
        "              ",
        "            ",
        "   .[||].   ",
        "  [ {E}  {E} ]  ",
        "  [ -=-= ]  ",
        "   `----´   ",
    ],
];

const PENGUIN_FRAMES: [[&str; 6]; 3] = [
    [
        "              ",
        "            ",
        "   .---.    ",
        "  ({E}>{E})    ",
        "  /(   )\\   ",
        "   `---´    ",
    ],
    [
        "              ",
        "            ",
        "   .---.    ",
        "  ({E}>{E})    ",
        " / (   )\\   ",
        "   `---´    ",
    ],
    [
        "              ",
        "            ",
        "   .---.    ",
        "  ({E}>{E})    ",
        "  /(   ) \\  ",
        "   `---´    ",
    ],
];

const GHOST_FRAMES: [[&str; 6]; 3] = [
    [
        "              ",
        "            ",
        "   .----.   ",
        "  / {E}  {E} \\  ",
        "  |      |  ",
        "  ~`~``~`~  ",
    ],
    [
        "              ",
        "            ",
        "   .----.   ",
        "  / {E}  {E} \\  ",
        "  |      |  ",
        "  `~~``~~`  ",
    ],
    [
        "              ",
        "            ",
        "    .----.  ",
        "   / {E}  {E} \\ ",
        "   |      | ",
        "   ~`~``~`~ ",
    ],
];

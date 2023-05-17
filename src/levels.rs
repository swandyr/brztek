pub mod commands;
mod draw;
pub mod handle_message;
pub mod queries;
pub mod user_level;
pub mod xp_func;

// Xp parameters
const MIN_XP_GAIN: i64 = 15;
const MAX_XP_GAIN: i64 = 25;
const DELAY_ANTI_SPAM: i64 = 60;

// Rank card constants
const CARD_FONT: &str = "Akira Expanded"; // Font needs to be installed on the system (https://www.dafont.com/akira-expanded.font)
const DEFAULT_PP_TESSELATION_VIOLET: &str = "assets/images/default-pp/Tessellation-Violet.png";
const TOP_TITLE_HEIGHT: usize = 60;
const TOP_USER_HEIGHT: usize = 32;

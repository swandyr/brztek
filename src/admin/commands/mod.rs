pub mod import_mee6_levels;
pub mod set_xp;
pub mod shutdown;

use crate::{Data, Error};

pub use import_mee6_levels::import_mee6_levels;
pub use set_xp::set_xp;
pub use shutdown::shutdown;

pub fn all() -> Vec<poise::Command<Data, Error>> {
    vec![import_mee6_levels::import_mee6_levels(), set_xp::set_xp()]
}

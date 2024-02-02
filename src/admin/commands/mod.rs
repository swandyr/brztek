pub mod import_mee6_levels;
pub mod selectmenu;
pub mod set_xp;

use crate::{Data, Error};

pub use import_mee6_levels::import_mee6_levels;
pub use selectmenu::selectmenu;
pub use set_xp::set_xp;

pub fn all() -> Vec<poise::Command<Data, Error>> {
    vec![
        import_mee6_levels::import_mee6_levels(),
        selectmenu::selectmenu(),
        set_xp::set_xp(),
    ]
}

mod bigrig;
mod clean;
mod learn;
mod learned;
mod ping;
mod setcolor;

use super::{consts, queries};
use crate::{Data, Error};

pub use bigrig::bigrig;
pub use clean::clean;
pub use learn::learn;
pub use learned::learned;
pub use ping::ping;
pub use setcolor::setcolor;

pub fn all() -> Vec<poise::Command<Data, Error>> {
    vec![
        bigrig::bigrig(),
        clean::clean(),
        learn::learn(),
        learned::learned(),
        ping::ping(),
        setcolor::setcolor(),
    ]
}

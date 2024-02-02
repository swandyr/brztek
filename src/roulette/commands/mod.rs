pub mod rffstar;
pub mod roulette;
pub mod statroulette;
pub mod toproulette;

use std::vec;

use super::{consts, func, models, queries};
use crate::{Data, Error};

pub use rffstar::rffstar;
pub use roulette::roulette;
pub use statroulette::statroulette;
pub use toproulette::toproulette;

pub fn all() -> Vec<poise::Command<Data, Error>> {
    vec![
        rffstar::rffstar(),
        roulette::roulette(),
        statroulette::statroulette(),
        toproulette::toproulette(),
    ]
}

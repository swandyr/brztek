pub mod rank;
pub mod top;

use super::{constants, draw, func, models, queries};
use crate::{Data, Error};

pub use rank::rank;
pub use top::top;

pub fn all() -> Vec<poise::Command<Data, Error>> {
    vec![rank::rank(), top::top()]
}

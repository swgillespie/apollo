// Copyright 2017-2019 Sean Gillespie.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.
use crate::position::Position;

mod score;
mod shannon_evaluator;

pub use score::Score;
pub use shannon_evaluator::ShannonEvaluator;

pub trait BoardEvaluator: Default {
    fn evaluate(&self, pos: &Position) -> Score;
}

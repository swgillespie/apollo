// Copyright 2017-2019 Sean Gillespie.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.
use std::time::Duration;

use crate::eval::Score;
use crate::moves::Move;
use crate::position::Position;
use crate::search::DataRecorder;

pub struct SearchResult {
    pub best_move: Move,
    pub nodes_searched: u64,
    pub score: Score,
}

pub trait Searcher {
    fn search(
        &mut self,
        pos: &Position,
        max_depth: u32,
        time_budget: Option<Duration>,
        data: &dyn DataRecorder,
    ) -> SearchResult;
}

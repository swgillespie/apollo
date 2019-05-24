// Copyright 2017-2019 Sean Gillespie.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

mod iterative_deepening_searcher;
mod searcher;
mod transposition_table;

pub use iterative_deepening_searcher::IterativeDeepeningSearcher;
pub use searcher::{NaiveSearcher, SearchResult, Searcher};
pub use transposition_table::{NodeKind, TableEntry, TableStats, TranspositionTable};

// Copyright 2017-2020 Sean Gillespie.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.
use serde_derive::{Deserialize, Serialize};

use crate::book::radix_tree::RadixTree;
use crate::moves::Move;

#[derive(Deserialize, Serialize)]
pub struct OpeningBook {
    tree: RadixTree<BookEntry>,
}

impl OpeningBook {
    pub fn book_moves(&self, played_sequence: &[Move]) -> Vec<Move> {
        let mut moves = vec![];
        self.tree.each_child(played_sequence, |mov, _| {
            moves.push(mov);
        });

        moves
    }
}

#[derive(Copy, Clone, Debug, Deserialize, Serialize)]
enum Category {
    A(u32),
    B(u32),
    C(u32),
    D(u32),
    E(u32),
}

#[derive(Debug, Deserialize, Serialize)]
struct BookEntry {
    pub category: Category,
    pub lead_name: String,
    pub response_name: Option<String>,
}

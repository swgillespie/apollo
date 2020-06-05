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

#[derive(Clone, Deserialize, Serialize)]
pub struct OpeningBook {
    tree: RadixTree<BookEntry>,
}

impl OpeningBook {
    pub fn new() -> OpeningBook {
        OpeningBook {
            tree: RadixTree::new(),
        }
    }

    pub fn is_in_book(&self, line: &[Move]) -> bool {
        self.tree.get(line).is_some()
    }

    pub fn book_moves(&self, played_sequence: &[Move]) -> Vec<(Move, BookEntry)> {
        let mut moves = vec![];
        self.tree.each_child(played_sequence, |mov, entry| {
            moves.push((mov, entry.cloned().unwrap()));
        });

        moves
    }

    pub fn add_entry(&mut self, sequence: &[Move], value: BookEntry) {
        self.tree.insert(sequence, value)
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct BookEntry {
    pub category: String,
    pub lead_name: String,
    pub response_name: Option<String>,
}

impl BookEntry {
    /// Returns whether or not this line is unorthodox ("wacky"). While it's useful to keep these
    /// in the book in case an opponent tries to pull one of these on us, it's best that we never
    /// play them ourselves.
    ///
    /// The ECO database has a categorization scheme where it assigns a category for every line. Abnormal
    /// lines are categorized together, so we can check them here.
    pub fn is_wacky(&self) -> bool {
        match &*self.category {
            // A00 lines are super weird, leading with things like a3, b3, d3, g4, etc.
            // We shouldn't ever play these unless we're trolling.
            "A00" => true,
            // B00 lines are wacky extensions of the king's pawn opening, also useful for trolling
            // but not much else.
            "B00" => true,
            // Everything else is mostly legit, we'll have to exclude stuff as the bot plays it and
            // we try to figure out what the fuck it was thinking.
            _ => false,
        }
    }
}

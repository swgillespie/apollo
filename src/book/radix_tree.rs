// Copyright 2017-2020 Sean Gillespie.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! The `radix_tree` module provides a prefix tree for sequences of moves. The objective of this
//! module is to provide a compact and efficiently-queryable data structure for move books,
//! particularly for opening books.
//!
//! In terms of implementation, `RadixTree<T>` provides a straightforward and uninteresting
//! implementation of a radix tree using moves as the prefix. It is somewhat unique in that it
//! is designed to be serialized and deserialized, so that programs can generate an opening book
//! and persist it on disk to be consumed at runtime by a chess engine.

use crate::moves::Move;
use serde_derive::{Deserialize, Serialize};
use std::collections::HashMap;

/// A simple and efficient radix tree keyed by sequences of moves.
#[derive(Clone, Deserialize, Serialize)]
pub struct RadixTree<T> {
    root: RadixTreeNode<T>,
}

impl<T: Clone> RadixTree<T> {
    pub fn new() -> RadixTree<T> {
        RadixTree {
            root: RadixTreeNode {
                key: Move::null(),
                key_str: "0000".to_owned(),
                value: None,
                children: HashMap::new(),
            },
        }
    }

    /// Inserts a value into the radix tree keyed by the given move sequence. Does not replace the
    /// entry if a value is already present in the radix tree with the same move sequence.
    pub fn insert(&mut self, move_sequence: &[Move], value: T) {
        let mut cursor = &mut self.root;
        for &mov in move_sequence.iter() {
            let child = cursor.children.entry(mov).or_insert_with(|| {
                Box::new(RadixTreeNode {
                    key: mov,
                    key_str: mov.as_uci(),
                    value: Some(value.clone()),
                    children: HashMap::new(),
                })
            });
            cursor = child;
        }

        if cursor.value.is_none() {
            cursor.value = Some(value);
        }
    }

    /// Retrieves a value from the radix tree keyed by the given move sequence.
    pub fn get(&self, move_sequence: &[Move]) -> Option<&T> {
        self.get_node(move_sequence).and_then(|n| n.value.as_ref())
    }

    pub fn each_child<F>(&self, move_sequence: &[Move], mut func: F)
    where
        F: FnMut(Move, Option<&T>),
    {
        if let Some(node) = self.get_node(move_sequence) {
            for (&mov, node) in node.children.iter() {
                func(mov, node.value.as_ref())
            }
        }
    }

    fn get_node(&self, move_sequence: &[Move]) -> Option<&RadixTreeNode<T>> {
        let mut cursor = &self.root;
        for mov in move_sequence.iter() {
            let child = match cursor.children.get(mov) {
                Some(child) => child,
                None => return None,
            };

            cursor = child;
        }

        Some(cursor)
    }
}

#[derive(Clone, Deserialize, Serialize)]
struct RadixTreeNode<T> {
    key: Move,
    key_str: String,
    value: Option<T>,
    children: HashMap<Move, Box<RadixTreeNode<T>>>,
}

#[cfg(test)]
mod tests {
    use super::RadixTree;
    use crate::moves::Move;
    use crate::types::Square;

    #[test]
    fn insert_get_smoke() {
        let mut tree = RadixTree::new();
        let moves = vec![Move::quiet(Square::E5, Square::E6)];
        tree.insert(&moves, 5);
        assert_eq!(tree.get(&moves).cloned().unwrap(), 5);
    }

    #[test]
    fn insert_common_prefix() {
        let mut tree = RadixTree::new();
        tree.insert(
            &[
                Move::quiet(Square::E5, Square::E6),
                Move::quiet(Square::E6, Square::E7),
            ],
            6,
        );

        tree.insert(
            &[
                Move::quiet(Square::E5, Square::E6),
                Move::quiet(Square::E6, Square::E8),
            ],
            7,
        );

        assert_eq!(
            tree.get(&[
                Move::quiet(Square::E5, Square::E6),
                Move::quiet(Square::E6, Square::E7),
            ])
            .cloned()
            .unwrap(),
            6
        );

        assert_eq!(
            tree.get(&[
                Move::quiet(Square::E5, Square::E6),
                Move::quiet(Square::E6, Square::E8),
            ])
            .cloned()
            .unwrap(),
            7
        );

        assert_eq!(
            tree.get(&[Move::quiet(Square::E5, Square::E6)])
                .cloned()
                .unwrap(),
            6
        );
    }
}

// Copyright 2017 Sean Gillespie.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.
use apollo_engine::{Position, Move};
use rand;

pub fn search_from_position(pos: &Position) -> Move {
    // for now - make a random move.
    let legal_moves = {
        let mut moves = vec![];
        for mov in pos.pseudolegal_moves() {
            let mut clone = pos.clone();
            clone.apply_move(mov);
            if !clone.is_check(pos.side_to_move()) {
                moves.push(mov);
            }
        }

        moves
    };

    assert!(legal_moves.len() != 0);
    let idx = rand::random::<usize>() % legal_moves.len();
    legal_moves[idx]
}
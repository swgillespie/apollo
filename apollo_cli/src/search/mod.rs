// Copyright 2017 Sean Gillespie.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.
use std::f64;
use cancellation::CancellationToken;
use apollo::{Position, Move};
use rand;

mod alpha_beta;
mod eval;

/// Searches for the best move starting at the given position and looking at
/// the given depth, with the given cancellation token. Callers can signal
/// search cancellation using the cancellation token, which will cause the
/// search to terminate as quickly as possible and return the best move
/// that it has seen so far.
pub fn search(pos: &Position, depth: u32, ct: &CancellationToken) -> (f64, Move) {
    let (mut best_score, mut best_move) = (-f64::INFINITY, Move::null());
    let side = pos.side_to_move();
    for mov in pos.pseudolegal_moves() {
        let mut pos_dup = pos.clone();
        pos_dup.apply_move(mov);
        if pos_dup.is_check(side) {
            continue;
        }

        // search_impl returns Ok if it wasn't canceled, and Err if it was.
        // we don't care which one it is here.
        let score = match alpha_beta::negamax(&pos_dup, side, -f64::INFINITY, f64::INFINITY, depth, ct) {
            Ok(s) => -s,
            Err(s) => -s,
        };

        if score > best_score {
            best_score = score;
            best_move = mov;
        }
    }

    (best_score, best_move)
}

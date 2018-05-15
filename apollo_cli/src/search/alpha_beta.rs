// Copyright 2017 Sean Gillespie.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Simple alpha-beta search for searching the move tree.
use search::eval;
use engine::trans_table;
use apollo::{Position, Move, Color};
use std::f64;
use cancellation::CancellationToken;

macro_rules! cancel_poll {
    ($ct:expr, $alpha:expr) => {
        if $ct.is_canceled() {
            return Err($alpha);
        }
    }
}

pub fn negamax(position: &Position,
               us: Color,
               mut alpha: f64,
               beta: f64,
               depth: u32,
               ct: &CancellationToken)
               -> Result<f64, f64> {
    //info!("search at position {:?}, depth {}, alpha {}, beta {}",
    //      position,
    //      depth,
    //      alpha,
    //      beta);
    
    if depth == 0 {
        return Ok(eval::evaluate(position, us));
    }

    cancel_poll!(ct, alpha);
    let mut best = -f64::INFINITY;
    let mut bail_for_cancel = false;
    for mov in position.pseudolegal_moves() {
        cancel_poll!(ct, alpha);
        let mut clone = position.clone();
        clone.apply_move(mov);
        if clone.is_check(us) {
            continue;
        }

        let score = match negamax(&clone, us, -beta, -alpha, depth - 1, ct) {
            Ok(score) => -score,
            Err(score) => {
                bail_for_cancel = true;
                -score
            }
        };

        best = best.max(score);
        alpha = alpha.max(score);
        if alpha >= beta || bail_for_cancel {
            break;
        }
    }

    Ok(best)
}

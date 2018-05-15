// Copyright 2017 Sean Gillespie.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.
use apollo::{Position, Color};

const KING_WEIGHT: f64 = 200.0;
const QUEEN_WEIGHT: f64 = 9.0;
const ROOK_WEIGHT: f64 = 5.0;
const BISHOP_WEIGHT: f64 = 3.0;
const KNIGHT_WEIGHT: f64 = 3.0;
const PAWN_WEIGHT: f64 = 1.0;
const MOBILITY_WEIGHT: f64 = 0.1;

/// Simple evaluation function that takes only material and mobility into
/// account.
pub fn evaluate(pos: &Position, us: Color) -> f64 {
    let scale = match us {
        Color::White => 1.0f64,
        Color::Black => -1.0f64,
    };

    let score =
        KING_WEIGHT * (pos.kings(Color::White).count() as i32 - pos.kings(Color::Black).count() as i32) as f64 +
        QUEEN_WEIGHT * (pos.queens(Color::White).count() as i32 - pos.queens(Color::Black).count() as i32) as f64 +
        ROOK_WEIGHT * (pos.rooks(Color::White).count() as i32 - pos.rooks(Color::Black).count() as i32) as f64+
        BISHOP_WEIGHT * (pos.bishops(Color::White).count() as i32 - pos.bishops(Color::Black).count() as i32) as f64 +
        KNIGHT_WEIGHT * (pos.knights(Color::White).count() as i32 - pos.knights(Color::Black).count() as i32) as f64+
        PAWN_WEIGHT * (pos.pawns(Color::White).count() as i32 - pos.pawns(Color::Black).count() as i32) as f64;

    let legal_moves = {
        let mut mov_count = 0;
        for mov in pos.pseudolegal_moves() {
            let mut clone = pos.clone();
            clone.apply_move(mov);
            if !clone.is_check(us) {
                mov_count += 1;
            }
        }

        mov_count as f64 * MOBILITY_WEIGHT
    };

    (legal_moves + score) * scale
}

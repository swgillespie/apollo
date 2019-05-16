// Copyright 2017-2019 Sean Gillespie.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.
use crate::analysis::Analysis;
use crate::eval::{BoardEvaluator, Score};
use crate::position::Position;
use crate::types::Color;

const KING_WEIGHT: f32 = 2000f32;
const QUEEN_WEIGHT: f32 = 9f32;
const ROOK_WEIGHT: f32 = 5f32;
const BISHOP_WEIGHT: f32 = 3f32;
const KNIGHT_WEIGHT: f32 = 3f32;
const PAWN_WEIGHT: f32 = 1f32;
const PAWN_FORMATION_WEIGHT: f32 = 0.5;
const MOBILITY_WEIGHT: f32 = 0.1;

pub struct ShannonEvaluator;

impl ShannonEvaluator {
    pub fn new() -> ShannonEvaluator {
        ShannonEvaluator
    }
}

impl Default for ShannonEvaluator {
    fn default() -> ShannonEvaluator {
        ShannonEvaluator
    }
}

impl BoardEvaluator for ShannonEvaluator {
    fn evaluate(&self, pos: &Position) -> Score {
        let analysis = Analysis::new(pos);

        // Check out mobility first - it's possible that a side has been checkmated.
        let white_mobility = analysis.mobility(Color::White);
        if white_mobility == 0 {
            if pos.is_check(Color::White) {
                return Score::Loss(0);
            } else {
                return Score::Evaluated(0f32);
            }
        }
        let black_mobility = analysis.mobility(Color::Black);
        if black_mobility == 0 {
            if pos.is_check(Color::Black) {
                return Score::Win(0);
            } else {
                return Score::Evaluated(0f32);
            }
        }

        let kings = evaluate_metric(KING_WEIGHT, |c| pos.kings(c).count() as f32);
        let queens = evaluate_metric(QUEEN_WEIGHT, |c| pos.queens(c).count() as f32);
        let rooks = evaluate_metric(ROOK_WEIGHT, |c| pos.rooks(c).count() as f32);
        let bishops = evaluate_metric(BISHOP_WEIGHT, |c| pos.bishops(c).count() as f32);
        let knights = evaluate_metric(KNIGHT_WEIGHT, |c| pos.knights(c).count() as f32);
        let pawns = evaluate_metric(PAWN_WEIGHT, |c| pos.pawns(c).count() as f32);
        let mobility = MOBILITY_WEIGHT * (white_mobility as f32 - black_mobility as f32);
        let isolated_pawns = evaluate_metric(PAWN_FORMATION_WEIGHT, |c| {
            analysis.isolated_pawns(c).count() as f32
        });
        let backward_pawns = evaluate_metric(PAWN_FORMATION_WEIGHT, |c| {
            analysis.backward_pawns(c).count() as f32
        });
        let doubled_pawns = evaluate_metric(PAWN_FORMATION_WEIGHT, |c| {
            analysis.doubled_pawns(c).count() as f32
        });

        Score::Evaluated(
            kings
                + queens
                + rooks
                + bishops
                + knights
                + pawns
                + isolated_pawns
                + backward_pawns
                + doubled_pawns
                + mobility,
        )
    }
}

fn evaluate_metric<F>(weight: f32, func: F) -> f32
where
    F: Fn(Color) -> f32,
{
    let white = func(Color::White);
    let black = func(Color::Black);
    weight * (white - black)
}

#[cfg(test)]
mod tests {
    use super::ShannonEvaluator;
    use crate::eval::{BoardEvaluator, Score};
    use crate::position::Position;

    #[test]
    fn white_mate_evaluation() {
        let eval = ShannonEvaluator::new();
        let pos = Position::from_fen("8/8/8/8/8/3k4/3q4/3K4 w - - 0 1").unwrap();
        assert_eq!(Score::Loss(0), eval.evaluate(&pos));
    }

    #[test]
    fn black_mate_evaluation() {
        let eval = ShannonEvaluator::new();
        let pos = Position::from_fen("4k3/4Q3/4K3/8/8/8/8/8 b - - 0 1").unwrap();
        assert_eq!(Score::Win(0), eval.evaluate(&pos));
    }
}

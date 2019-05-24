// Copyright 2017-2019 Sean Gillespie.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.
use std::time::Duration;

use crate::eval::{BoardEvaluator, Score};
use crate::move_generator::{MoveGenerator, MoveVec};
use crate::moves::Move;
use crate::position::Position;
use crate::types::Color;

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
    ) -> SearchResult;
}

pub struct NaiveSearcher<E> {
    evaluator: E,
    nodes_searched: u64,
    moving_player: Color,
}

impl<E: BoardEvaluator> NaiveSearcher<E> {
    pub fn new() -> NaiveSearcher<E> {
        NaiveSearcher {
            evaluator: Default::default(),
            nodes_searched: 0u64,
            moving_player: Color::White,
        }
    }

    fn alpha_beta(&mut self, pos: &Position, mut alpha: Score, beta: Score, depth: u32) -> Score {
        if depth == 0 {
            return self.quiesce(pos, alpha, beta);
        }

        // TODO(sean) clean this up - should mates be in moves or plies? currently moves, because
        // this is what UCI wants.
        let is_ply = true;
        // let is_ply = pos.side_to_move() != self.moving_player;
        let gen = MoveGenerator::new();
        let mut moves = MoveVec::default();
        gen.generate_moves(pos, &mut moves);

        let mut candidate_positions: Vec<Position> = moves
            .into_iter()
            .filter(|&m| pos.is_legal_given_pseudolegal(m))
            .map(|m| {
                let mut cloned = pos.clone();
                cloned.apply_move(m);
                cloned
            })
            .collect();
        candidate_positions.sort_by_cached_key(|p| self.evaluator.evaluate(p));
        if candidate_positions.len() == 0 {
            // no moves - are we in check?
            if pos.is_check(pos.side_to_move()) {
                // we lost.
                return Score::Loss(0).step();
            } else {
                // we've drawn.
                return Score::Evaluated(0.0f32);
            }
        }
        for pos in candidate_positions {
            let score = -self.alpha_beta(&pos, -beta, -alpha, depth - 1);
            if score >= beta {
                return beta.step_if(is_ply);
            }

            if score > alpha {
                alpha = score;
            }
        }

        alpha.step_if(is_ply)
    }

    fn quiesce(&mut self, pos: &Position, _alpha: Score, _beta: Score) -> Score {
        self.nodes_searched += 1;
        let value = self.evaluator.evaluate(pos);
        match pos.side_to_move() {
            Color::White => value,
            Color::Black => -value,
        }
    }
}

impl<E: BoardEvaluator> Searcher for NaiveSearcher<E> {
    fn search(
        &mut self,
        pos: &Position,
        max_depth: u32,
        time_budget: Option<Duration>,
    ) -> SearchResult {
        assert!(
            time_budget.is_none(),
            "NaiveSearcher does not support time budgets"
        );

        self.nodes_searched = 0;
        self.moving_player = pos.side_to_move();
        let mut best_move = None;
        let mut best_score = Score::Loss(0);

        let gen = MoveGenerator::new();
        let mut moves = MoveVec::default();
        gen.generate_moves(pos, &mut moves);

        let mut candidate_positions: Vec<(Move, Position)> = moves
            .into_iter()
            .filter(|&m| pos.is_legal_given_pseudolegal(m))
            .map(|m| {
                let mut cloned = pos.clone();
                cloned.apply_move(m);
                (m, cloned)
            })
            .collect();
        candidate_positions.sort_by_cached_key(|(_, p)| self.evaluator.evaluate(p));

        let mut alpha = Score::Loss(0);
        let beta = Score::Win(0);
        for (mov, pos) in candidate_positions {
            let score = -self.alpha_beta(&pos, -beta, -alpha, max_depth - 1);
            if score > alpha {
                alpha = score;
            }

            if score > best_score || best_move.is_none() {
                best_score = score;
                best_move = Some(mov)
            }
        }

        SearchResult {
            best_move: best_move.unwrap_or(Move::null()),
            score: best_score,
            nodes_searched: self.nodes_searched,
        }
    }
}

// Copyright 2017-2019 Sean Gillespie.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.
use std::time::{Duration, Instant};

use crate::eval::{BoardEvaluator, Score};
use crate::move_generator::{MoveGenerator, MoveVec};
use crate::moves::Move;
use crate::position::Position;
use crate::search::{NodeKind, Record, SearchResult, Searcher, TranspositionTable};
use crate::types::Color;

pub struct IterativeDeepeningSearcher<E> {
    evaluator: E,
    ttable: TranspositionTable,
}

impl<E: Default> IterativeDeepeningSearcher<E> {
    pub fn new() -> IterativeDeepeningSearcher<E> {
        IterativeDeepeningSearcher {
            evaluator: Default::default(),
            ttable: TranspositionTable::new(),
        }
    }
}

impl<E: BoardEvaluator> Searcher for IterativeDeepeningSearcher<E> {
    fn search(
        &mut self,
        pos: &Position,
        max_depth: u32,
        time_budget: Option<Duration>,
    ) -> SearchResult {
        let mut search = IterativeSearch::new(self, max_depth, time_budget);
        search.search(pos)
    }
}

impl<E: Default> Default for IterativeDeepeningSearcher<E> {
    fn default() -> IterativeDeepeningSearcher<E> {
        IterativeDeepeningSearcher::new()
    }
}

struct IterativeSearch<'a, E> {
    searcher: &'a IterativeDeepeningSearcher<E>,
    max_depth: u32,
    time_budget: Option<Duration>,
    start_time: Instant,

    stats: Record,
}

impl<'a, E: BoardEvaluator> IterativeSearch<'a, E> {
    pub fn new(
        searcher: &'a IterativeDeepeningSearcher<E>,
        max_depth: u32,
        budget: Option<Duration>,
    ) -> IterativeSearch<'a, E> {
        IterativeSearch {
            searcher: searcher,
            max_depth: max_depth,
            time_budget: budget,
            start_time: Instant::now(),
            stats: Default::default(),
        }
    }

    /// Does a toplevel search of a given depth.
    fn search_depth(&mut self, pos: &Position, depth: u32) -> SearchResult {
        let alpha = Score::Loss(0);
        let beta = Score::Win(0);
        let score = self.alpha_beta(pos, alpha, beta, depth);
        let best_move = self.searcher.ttable.query(pos, |entry| {
            entry
                .expect("search_depth yielded t-table miss after search")
                .best_move
                .expect("search_depth thinks that root node is an all-node")
        });

        SearchResult {
            best_move: best_move,
            score: score,
            nodes_searched: self.stats.nodes,
        }
    }

    fn quiesce(&mut self, pos: &Position, _alpha: Score, _beta: Score) -> Score {
        self.stats.nodes += 1;
        let value = self.searcher.evaluator.evaluate(pos);
        match pos.side_to_move() {
            Color::White => value,
            Color::Black => -value,
        }
    }

    fn alpha_beta(&mut self, pos: &Position, mut alpha: Score, beta: Score, depth: u32) -> Score {
        //debug!("{}", pos.as_fen());
        debug!("depth: {}", depth);
        debug!("alpha: {}", alpha);
        debug!("beta:  {}", beta);
        if depth == 0 {
            debug!("quiescing due to depth 0");
            return self.quiesce(pos, alpha, beta);
        }

        // The alpha-beta function in this searcher is designed to exploit the transposition table to take the best
        // known path through the game tree. The transposition table serves two purposes:
        //   1. If the t-table records that we've already done a really deep search for a particular position, we can
        //      use the t-table's exact results as the results of this search and avoid having to do a search entirely.
        //   2. If the t-table records that we've done a search for this position, but it's not deep enough to serve
        //      this search, we can use its best move (or "hash move") to guide our search. We'll search that move
        //      before even generating moves for the current position, in the hopes that the hash move either fails high
        //      or produces a really high alpha.
        let hash_move = if let Some(entry) = self.searcher.ttable.query_copy(pos) {
            // Transposition table hit. We might not be able to use this hit, though:
            //    1. If the entry's depth is less than the depth we are currently searching at, we shouldn't
            //       use this entry since the search we are about to do is going to be higher fidelity.
            //    2. If the entry's best move isn't a legal move, then we probably had a collision in the t-table
            //       and shouldn't use it.
            //    3. If the entry is an all node, it doesn't even have a hash move. We can still try to fail low, but
            //       we won't get a hash move out of it.

            debug!("tt hit! {:?}, search depth {}", entry, depth);
            let hash_move = entry.best_move;
            if entry.depth >= depth && (hash_move.is_none() || pos.is_legal(hash_move.unwrap())) {
                match entry.node {
                    NodeKind::PrincipalVariation(score) => {
                        // The last time we searched at this depth or greater, this move was a PV-node. This is the
                        // best case scenario; we know exactly what the score is. We don't have to search this subtree
                        // at all.
                        debug!("exiting with score {} due to TT PV hit", score.step());
                        return score.step();
                    }
                    NodeKind::Cut(score) => {
                        // The last time we searched at this depth or greater, this move caused a beta cutoff. The score
                        // here is a lower-bound on the exact score of the node.
                        //
                        // If the lower bound is greater than beta, we don't need to search this node and can instead
                        // return beta.
                        if score >= beta {
                            debug!(
                                "exiting with score {} due to TT hit beta cutoff",
                                beta.step()
                            );
                            return beta.step();
                        }

                        // If the lower bound is greater than alpha, bump up alpha to match.
                        if score >= alpha {
                            debug!("bumping alpha up to {} due to TT hit", score);
                            alpha = score;
                        }

                        // Otherwise, we should search the hash move first - it'll probably cause a beta cutoff.
                    }
                    NodeKind::All(score) => {
                        // The last time we searched at this depth or greater, we searched all children of this node and
                        // none of them improved alpha. The score here is an upper-bound on the exact score of the node.
                        //
                        // If the upper bound is worse than alpha, we're not going to find anything better if we search
                        // here.
                        if score <= alpha {
                            debug!(
                                "exiting with score {} due to TT hit alpha cutoff",
                                alpha.step()
                            );
                            return alpha.step();
                        }

                        // Otherwise, we'll need to search everything, starting at the hash move.
                    }
                }

                hash_move
            } else {
                None
            }
        } else {
            None
        };

        let mut improved_alpha = false;

        // Before generating moves, if there is a hash move, try it and see if it fails high.
        if let Some(hash_move) = hash_move {
            debug!("inspecting hash move {} for cutoffs", hash_move);
            debug_assert!(pos.is_legal(hash_move));
            let mut hash_pos = pos.clone();
            hash_pos.apply_move(hash_move);
            let score = -self.alpha_beta(&hash_pos, -beta, -alpha, depth - 1);
            if score >= beta {
                self.searcher
                    .ttable
                    .record_cut(pos, hash_move, depth, score);
                return beta.step();
            }

            if score > alpha {
                improved_alpha = true;
                debug!(
                    "hash move {} improved PV, setting alpha = {}",
                    hash_move, score
                );
                self.searcher
                    .ttable
                    .record_principal_variation(pos, hash_move, depth, score);
                alpha = score;
            }
        }

        if self.out_of_time() {
            debug!("bailing due to being out of time");
            return alpha;
        }

        // We didn't hit the TT, or we hit the TT and the hash move didn't cause a cutoff. We'll have to do a full move
        // search.

        debug!("generating moves");
        let gen = MoveGenerator::new();
        let mut moves = MoveVec::default();
        gen.generate_moves(pos, &mut moves);
        moves.retain(|&mut m| pos.is_legal_given_pseudolegal(m));
        if moves.len() == 0 {
            // No legal moves available. Are we in check?
            let score = if pos.is_check(pos.side_to_move()) {
                // We lost.
                Score::Loss(0)
            } else {
                // We've drawn.
                Score::Evaluated(0.0f32)
            };

            //debug!("{} is checkmate or draw position", pos.as_fen());
            self.searcher
                .ttable
                .record_principal_variation(pos, Move::null(), depth, score);
            return score.step();
        }

        for mov in moves {
            let mut child = pos.clone();
            child.apply_move(mov);
            let score = -self.alpha_beta(&child, -beta, -alpha, depth - 1);
            if score >= beta {
                self.searcher.ttable.record_cut(pos, mov, depth, score);
                return beta.step();
            }

            if score > alpha {
                improved_alpha = true;
                self.searcher
                    .ttable
                    .record_principal_variation(pos, mov, depth, score);
                alpha = score;
            }
        }

        if !improved_alpha {
            //debug!("recording {} as all node", pos.as_fen());
            self.searcher.ttable.record_all(pos, depth, alpha);
        }

        alpha.step()
    }

    fn search(&mut self, pos: &Position) -> SearchResult {
        let mut current_best_move = Move::null();
        let mut current_best_score = Score::Loss(0);
        for depth in 1..=self.max_depth {
            debug!("beginning search of depth {}", depth);
            let result = self.search_depth(pos, depth);
            if self.out_of_time() {
                return SearchResult {
                    best_move: current_best_move,
                    score: current_best_score,
                    nodes_searched: self.stats.nodes,
                };
            }

            current_best_move = result.best_move;
            current_best_score = result.score;
            info!("pv ({}): {:?}", current_best_score, self.get_pv(pos, depth));
        }

        SearchResult {
            best_move: current_best_move,
            score: current_best_score,
            nodes_searched: self.stats.nodes,
        }
    }

    fn get_pv(&self, pos: &Position, depth: u32) -> Vec<Move> {
        let mut pv = vec![];
        let mut pv_clone = pos.clone();
        for _ in 0..depth {
            let best_move = self
                .searcher
                .ttable
                .query(&pv_clone, |e| e.and_then(|e| e.best_move));
            if let Some(best_move) = best_move {
                pv.push(best_move);
                pv_clone.apply_move(best_move);
            } else {
                break;
            }
        }

        pv
    }

    fn out_of_time(&self) -> bool {
        if let Some(budget) = self.time_budget {
            let start = self.start_time;
            let now = Instant::now();
            now - start > budget
        } else {
            false
        }
    }
}

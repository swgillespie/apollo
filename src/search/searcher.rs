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
use crate::search::{DataRecorder, NodeKind, Record, TranspositionTable};
use crate::types::{Color, PieceKind, Square};

pub struct SearchResult {
    pub best_move: Move,
    pub nodes_searched: u64,
    pub score: Score,
}

pub struct Searcher<E> {
    evaluator: E,
    ttable: TranspositionTable,
}

impl<E: BoardEvaluator> Searcher<E> {
    pub fn new() -> Searcher<E> {
        Searcher {
            evaluator: Default::default(),
            ttable: TranspositionTable::new(),
        }
    }

    pub fn search(
        &mut self,
        pos: &Position,
        max_depth: u32,
        time_budget: Option<Duration>,
        recorder: &dyn DataRecorder,
    ) -> SearchResult {
        let mut search = IterativeSearch::new(self, max_depth, time_budget);
        search.search(pos, recorder)
    }
}

impl<E: BoardEvaluator> Default for Searcher<E> {
    fn default() -> Searcher<E> {
        Searcher::new()
    }
}

struct IterativeSearch<'a, E> {
    searcher: &'a Searcher<E>,
    max_depth: u32,
    time_budget: Option<Duration>,
    start_time: Instant,

    stats: Record,
}

impl<'a, E: BoardEvaluator> IterativeSearch<'a, E> {
    pub fn new(
        searcher: &'a Searcher<E>,
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
    fn search_depth(
        &mut self,
        pos: &Position,
        depth: u32,
        recorder: &dyn DataRecorder,
    ) -> SearchResult {
        self.stats = Default::default();
        self.stats.depth = depth;
        let alpha = Score::Loss(0);
        let beta = Score::Win(0);
        let score = self.alpha_beta(pos, alpha, beta, depth);
        let best_move = self.searcher.ttable.query(pos, |entry| {
            entry
                .expect("search_depth yielded t-table miss after search")
                .best_move
                .expect("search_depth thinks that root node is an all-node")
        });

        recorder.record(pos, &self.stats);
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

    fn consider_transposition(
        &mut self,
        pos: &Position,
        alpha: &mut Score,
        beta: Score,
        depth: u32,
    ) -> (Option<Move>, Option<Score>) {
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
            self.stats.tt_absolute_hit += 1;
            let hash_move = entry.best_move;
            if entry.depth >= depth && (hash_move.is_none() || pos.is_legal(hash_move.unwrap())) {
                match entry.node {
                    NodeKind::PrincipalVariation(score) => {
                        // The last time we searched at this depth or greater, this move was a PV-node. This is the
                        // best case scenario; we know exactly what the score is. We don't have to search this subtree
                        // at all.
                        debug!("exiting with score {} due to TT PV hit", score.step());
                        self.stats.tt_absolute_hit_pv += 1;
                        return (hash_move, Some(score.step()));
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
                            self.stats.tt_absolute_hit_cut += 1;
                            return (hash_move, Some(beta.step()));
                        }

                        // If the lower bound is greater than alpha, bump up alpha to match.
                        if score >= *alpha {
                            debug!("bumping alpha up to {} due to TT hit", score);
                            self.stats.tt_absolute_hit_cut_improved_alpha += 1;
                            *alpha = score;
                        }

                        // Otherwise, we should search the hash move first - it'll probably cause a beta cutoff.
                    }
                    NodeKind::All(score) => {
                        // The last time we searched at this depth or greater, we searched all children of this node and
                        // none of them improved alpha. The score here is an upper-bound on the exact score of the node.
                        //
                        // If the upper bound is worse than alpha, we're not going to find anything better if we search
                        // here.
                        if score <= *alpha {
                            debug!(
                                "exiting with score {} due to TT hit alpha cutoff",
                                alpha.step()
                            );
                            self.stats.tt_absolute_hit_all += 1;
                            return (hash_move, Some(alpha.step()));
                        }

                        // Otherwise, we'll need to search everything, starting at the hash move.
                    }
                }

                hash_move
            } else {
                hash_move
            }
        } else {
            None
        };

        (hash_move, None)
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

        // Consult the transposition table. Have we seen this position before and, if so, does it produce a cutoff?
        // If so, there's no need to continue processing this position.
        let (mut hash_move, cutoff_score) =
            self.consider_transposition(pos, &mut alpha, beta, depth);
        if let Some(cutoff) = cutoff_score {
            return cutoff;
        }

        // Even if we didn't get a cutoff from the transposition table, we can at least begin the search with
        // the hash move.
        //
        // If we received a hash move, it might not be legal (from a hash collision). Apply a legality test
        // before proceeding.
        hash_move = hash_move.and_then(|mov| if pos.is_legal(mov) { Some(mov) } else { None });

        // Keep track if any move improved alpha. If so, this is a PV node.
        let mut improved_alpha = false;

        // Before generating moves, if there is a hash move, try it and see if it fails high.
        if let Some(hash_move) = hash_move {
            debug!("inspecting hash move {} for cutoffs", hash_move);
            debug_assert!(pos.is_legal(hash_move));
            self.stats.hash_move_node += 1;
            let mut hash_pos = pos.clone();
            hash_pos.apply_move(hash_move);
            let score = -self.alpha_beta(&hash_pos, -beta, -alpha, depth - 1);
            if score >= beta {
                self.searcher
                    .ttable
                    .record_cut(pos, hash_move, depth, score);
                self.stats.hash_move_beta_cutoff += 1;
                return beta.step();
            }

            if score > alpha {
                improved_alpha = true;
                debug!(
                    "hash move {} improved PV, setting alpha = {}",
                    hash_move, score
                );
                self.stats.hash_move_improved_alpha += 1;
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
        // Order our moves to favor good ones earlier.
        order_moves(pos, &mut moves);
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
            self.stats.pv_nodes += 1;
            return score.step();
        }

        for mov in moves {
            let mut child = pos.clone();
            child.apply_move(mov);
            let score = -self.alpha_beta(&child, -beta, -alpha, depth - 1);
            if score >= beta {
                self.searcher.ttable.record_cut(pos, mov, depth, score);
                self.stats.cut_nodes += 1;
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
            self.stats.all_nodes += 1;
            self.searcher.ttable.record_all(pos, depth, alpha);
        } else {
            self.stats.pv_nodes += 1;
        }

        alpha.step()
    }

    fn search(&mut self, pos: &Position, recorder: &dyn DataRecorder) -> SearchResult {
        let mut current_best_move = Move::null();
        let mut current_best_score = Score::Loss(0);
        for depth in 1..=self.max_depth {
            debug!("beginning search of depth {}", depth);
            let result = self.search_depth(pos, depth, recorder);
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

/// Performs move ordering for a list of legal moves from a given position. Move ordering is crucial
/// for alpha-beta search. It is our best defense against combinatorial explosion of the state space
/// of chess.
///
/// This function heuristically orders all moves in order of how good they appear to be, without searching
/// the tree of moves directly.
///
/// Note that the hash move is not included here, since the searcher handles that already.
fn order_moves(pos: &Position, moves: &mut [Move]) {
    // For the purposes of move ordering, we derive a total order of moves by ranking them
    // by their static exchange scores. Static exchange generally refers to captures, but for move
    // ordering we'll also consider promotions to count for a score.
    //
    // We'll drive a move score for every move and use that as the sorting key.
    fn move_score(pos: &Position, mov: Move) -> i32 {
        match mov {
            // En-passant is an annoying edge case in everything, SEE is no exception. Put it before
            // the quiet moves but don't consider it particularly good.
            mov if mov.is_en_passant() => 1,
            // TODO(swgillespie) - This probably overestimates the value of promotion captures...
            mov if mov.is_capture() && mov.is_promotion() => {
                mov.promotion_piece().value() - 1
                    + static_exchange_evaluation(pos, mov.destination())
            }
            mov if mov.is_capture() => static_exchange_evaluation(pos, mov.destination()),
            mov if mov.is_promotion() => mov.promotion_piece().value() - 1,
            _ => 0,
        }
    }

    moves.sort_by_cached_key(|&mov| -move_score(pos, mov));
}

fn static_exchange_evaluation(pos: &Position, target: Square) -> i32 {
    let mut value = 0;
    if let Some(attacker) = smallest_attacker(pos, target) {
        let target_piece = pos.piece_at(target).unwrap();
        let mut child = pos.clone();
        let mov = Move::capture(attacker, target);
        child.apply_move(mov);
        value = target_piece.kind.value() - static_exchange_evaluation(&child, target);
    }

    value
}

fn smallest_attacker(pos: &Position, target: Square) -> Option<Square> {
    let attackers = pos.squares_attacking(pos.side_to_move(), target);
    if attackers.empty() {
        return None;
    }

    let mut values: Vec<(Square, PieceKind)> = attackers
        .into_iter()
        .map(|sq| (sq, pos.piece_at(sq).unwrap().kind))
        .collect();

    values.sort_by_key(|(_, kind)| kind.value());
    return values.first().map(|(sq, _)| sq).cloned();
}

#[cfg(test)]
mod tests {
    use crate::eval::ShannonEvaluator;
    use crate::move_generator::{MoveGenerator, MoveVec};
    use crate::moves::Move;
    use crate::position::Position;
    use crate::search::NullDataRecorder;
    use crate::types::Square;

    use super::Searcher;
    use super::{order_moves, static_exchange_evaluation};

    #[test]
    // Test to ensure that we don't regress our alpha-beta prune too badly.
    fn opening_position_prune() {
        let pos = Position::from_start_position();
        let mut search: Searcher<ShannonEvaluator> = Default::default();
        let result = search.search(&pos, 2, None, &NullDataRecorder);
        assert!(result.nodes_searched <= 80);
    }

    #[test]
    fn see_pawn_exchange_bad_for_player() {
        let pos = Position::from_fen("8/6p1/1R3b2/8/8/2B5/8/5r2 w - - 0 1").unwrap();
        // White to move, white threatens f6 and initiates an exchange.
        let predicted_yield = static_exchange_evaluation(&pos, Square::F6);

        // White trades a bishop and a rook (8) for a pawn and a bishop (4), a loss of 4.
        assert_eq!(predicted_yield, -4);
    }

    #[test]
    fn see_exchange_good_for_player() {
        let pos = Position::from_fen("8/r2q4/8/8/6B1/8/3Q4/8 w - - 0 1").unwrap();
        // White to move, white threatens Bxd7 and initiates an exchange.
        let predicted_yield = static_exchange_evaluation(&pos, Square::D7);

        // White trades a bishop (3) for a queen and a rook (14), for a win of 11.
        assert_eq!(predicted_yield, 11);
    }

    #[test]
    fn move_ordering_good_captures_first() {
        let pos = Position::from_fen("5b2/8/3r2r1/2P5/5B2/8/3Q4/8 w - - 0 1").unwrap();
        let gen = MoveGenerator::new();
        let mut moves = MoveVec::default();
        gen.generate_moves(&pos, &mut moves);
        moves.retain(|&mut m| pos.is_legal_given_pseudolegal(m));

        order_moves(&pos, &mut moves);
        assert_eq!(
            moves.first().cloned().unwrap(),
            Move::capture(Square::C5, Square::D6)
        );
    }
}

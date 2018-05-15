// Copyright 2017 Sean Gillespie.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.
mod threads;
pub mod trans_table;

use std::thread;
use std::sync::Arc;
use parking_lot::RwLock;
use apollo::{self, Position, Move};
use uci;

pub(in engine) static CURRENT_POS : RwLock<Position> = RwLock::new(Position::new());

#[derive(Clone, Debug, Default)]
pub struct SearchRequest {
    pub depth: u32,
    pub starting_moves: Vec<Move>,
    pub ponder: bool,
    pub wtime: u32,
    pub btime: u32,
    pub winc: u32,
    pub binc: u32,
    pub moves_to_go: u32,
    pub nodes: u64,
    pub mate: bool,
    pub movetime: u32,
    pub infinite: bool
}

impl SearchRequest {
    pub fn new() -> SearchRequest {
        Default::default()
    }
}

pub fn initialize() {
    // initialize the engine state. This involves initializing
    // apollo_engine itself as well as initializing search thread(s).
    apollo::initialize();

    // initialize search threads
    threads::initialize();

    // initialize transposition table
    trans_table::initialize();
}

pub fn shutdown() {
    threads::shutdown();
}

// Indicate that we are going to start searching from a new game,
// so we need to clear all of our state.
pub fn new_game() {
    info!("clearing state for new game");
    trans_table::clear();
}

pub fn new_position(fen: &str, moves: &[&str]) {
    let mut pos = match Position::from_fen(fen) {
        Ok(pos) => pos,
        Err(_) => {
            warn!("invalid fen string: {}", fen);
            return;
        }
    };

    info!("setting position: {}", fen);
    if !moves.is_empty() {
        for movstr in moves {
            if let Some(mov) = Move::from_uci(&movstr) {
                info!("applying move: {}", movstr);
                pos.apply_move(mov);
            } else {
                warn!("ignoring invalid move: {}", movstr);
            }
        }
    }

    *CURRENT_POS.write() = pos;
}

pub fn go(req: &SearchRequest) {
    let request = Arc::new(req.clone());
    threads::request_search(request);
    thread::spawn(move || {
        let (score, mov) = threads::request_results();
        info!("best move: {} (score: {})", mov.as_uci(), score);
        uci::bestmove(mov);
        CURRENT_POS.write().apply_move(mov);
    });
}

pub fn stop() {
    threads::cancel_search();
}

pub fn current_position() -> Position {
    CURRENT_POS.read().clone()
}

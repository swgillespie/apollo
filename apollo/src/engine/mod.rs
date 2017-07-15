// Copyright 2017 Sean Gillespie.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.
mod threads;
mod trans_table;

use std::ops::Deref;
use parking_lot::RwLock;
use apollo_engine::{Position, Move};
use search;
use uci;
use apollo_engine;

static CURRENT_POS : RwLock<Position> = RwLock::new(Position::new());

pub fn initialize() {
    // initialize the engine state. This involves initializing
    // apollo_engine itself as well as initializing search thread(s).
    apollo_engine::initialize();

    // initialize search threads
    threads::initialize();

    // initialize transposition table
    trans_table::initialize();
}

// Indicate that we are going to start searching from a new game,
// so we need to clear all of our state.
pub fn new_game() {
    // we have no state yet :v)
    info!("clearing state for new game");
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

pub fn go() {
    let mov = {
        let pos = CURRENT_POS.read();
        search::search_from_position(pos.deref())
    };

    uci::bestmove(mov);
    CURRENT_POS.write().apply_move(mov);
}

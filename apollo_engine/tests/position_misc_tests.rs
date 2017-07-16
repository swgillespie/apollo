// Copyright 2017 Sean Gillespie.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.
extern crate apollo_engine;

use apollo_engine::{Position, Square, Move};

// it's kinda hard to test a hash function, but here's
// a smoke test regardless

#[test]
fn zobrist_smoke() {
    apollo_engine::initialize();
    let mut pos = Position::from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1")
        .unwrap();
    
    let hash = pos.hash();    
    assert_ne!(0, hash); 

    pos.apply_move(Move::quiet(Square::E2, Square::E4));
    assert_ne!(hash, pos.hash());
}
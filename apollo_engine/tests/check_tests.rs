// Copyright 2017 Sean Gillespie.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.
extern crate apollo_engine;
use apollo_engine::{Position, Color};

#[test]
fn smoke_test_starting_position() {
    apollo_engine::initialize();
    let pos = Position::from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1").unwrap();

    // white is not in check.
    assert!(!pos.is_check(Color::White));
}

#[test]
fn fools_mate_check() {
    apollo_engine::initialize();
    let pos = Position::from_fen("rnb1kbnr/pppp1ppp/8/4p3/6Pq/5P2/PPPPP2P/RNBQKBNR w KQkq - 0 1").unwrap();

    // white is checkmated
    assert!(pos.is_check(Color::White));
}

#[test]
fn sliding_piece_pin() {
    apollo_engine::initialize();
    let pos = Position::from_fen("8/8/4q3/8/8/8/4P3/4K3 w - - 0 1").unwrap();

    // white is not checked, the white pawn is blocking the queen
    assert!(!pos.is_check(Color::White));
}

#[test]
fn position_5_bug_1_absolute_pin() {
    apollo_engine::initialize();
    let pos = Position::from_fen("rnR2k1r/pp1qbppp/2p5/8/2B5/8/PPP1NnPP/RNBQK2R w KQ - 0 1").unwrap();

    // black is checked by the white rook
    assert!(pos.is_check(Color::Black));
}
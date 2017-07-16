// Copyright 2017 Sean Gillespie.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.
extern crate apollo_engine;
use apollo_engine::{Position, Color, Square, Move};

#[test]
fn smoke_test_starting_position() {
    apollo_engine::initialize();
    let pos = Position::from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1")
        .unwrap();

    // white is not in check.
    assert!(!pos.is_check(Color::White));
}

#[test]
fn fools_mate_check() {
    apollo_engine::initialize();
    let pos = Position::from_fen("rnb1kbnr/pppp1ppp/8/4p3/6Pq/5P2/PPPPP2P/RNBQKBNR w KQkq - 0 1")
        .unwrap();

    // white is checkmated
    assert!(pos.is_check(Color::White));
    assert!(pos.is_checkmate());
    assert!(pos.is_game_over());
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
    let pos = Position::from_fen("rnR2k1r/pp1qbppp/2p5/8/2B5/8/PPP1NnPP/RNBQK2R w KQ - 0 1")
        .unwrap();

    // black is checked by the white rook
    assert!(pos.is_check(Color::Black));
}

#[test]
fn stalemate_smoke() {
    apollo_engine::initialize();
    let pos = Position::from_fen("7k/5K2/6Q1/8/8/8/8/8 b - - 0 1").unwrap();

    // black's turn to move. black is not in check but black has no legal moves.
    assert!(pos.is_game_over());
    assert!(!pos.is_checkmate());
    assert!(pos.is_draw());
    assert!(pos.is_stalemate());
}

#[test]
fn fifty_move_rule_smoke() {
    apollo_engine::initialize();
    let mut pos = Position::from_fen("8/7k/6R1/5K2/1r3B2/8/8/8 w - - 49 121").unwrap();

    // white's turn to move, the game is not over
    assert!(!pos.is_game_over());
    pos.apply_move(Move::quiet(Square::G6, Square::G5));

    // black's turn to move, black can claim a draw.
    assert!(pos.is_game_over());
    assert!(pos.is_draw());
    assert!(pos.is_fifty_move_rule());
}
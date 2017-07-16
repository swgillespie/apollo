// Copyright 2017 Sean Gillespie.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.
#![feature(test)]

extern crate apollo_engine;
extern crate test;

use apollo_engine::{Position, Color};
use test::Bencher;

#[bench]
fn starting_position_check_bench(b: &mut Bencher) {
    apollo_engine::initialize();
    let pos = Position::from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1").unwrap();
    b.iter(|| pos.is_check(Color::White));
}

#[bench]
fn fools_mate_checkmate_bench(b: &mut Bencher) {
    // this benchmark is a reminder that testing for checkmate is potentially
    // expensive.
    apollo_engine::initialize();
    let pos = Position::from_fen("rnb1kbnr/pppp1ppp/8/4p3/6Pq/5P2/PPPPP2P/RNBQKBNR w KQkq - 0 1").unwrap();
    b.iter(|| pos.is_checkmate());
}

#[bench]
fn midgame_negative_checkmate_bench(b: &mut Bencher) {
    // this benchmark is a reminder of the cost for testing for checkmate
    // when the player is in check but not checkmated.
    apollo_engine::initialize();
    let pos = Position::from_fen("r3k1nr/1b1p2pp/p2b2Q1/n3p1B1/2B5/2N2N2/PP3PPP/R3R1K1 b kq - 0 1").unwrap();
    b.iter(|| pos.is_checkmate());
}
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

use apollo_engine::Position;
use test::Bencher;

#[bench]
fn starting_position_check_bench(b: &mut Bencher) {
    apollo_engine::initialize();
    let pos = Position::from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1").unwrap();
    b.iter(|| pos.is_check());
}
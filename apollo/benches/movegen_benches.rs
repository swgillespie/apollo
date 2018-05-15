// Copyright 2017 Sean Gillespie.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.
#![feature(test)]

extern crate apollo;
extern crate test;

use apollo::Position;
use test::Bencher;

#[bench]
fn midgame_move_generation(b: &mut Bencher) {
    apollo::initialize();
    let pos = Position::from_fen("r3k1nr/pb1p1ppp/2nb2q1/1B6/3QP3/2N2N2/PP3PPP/R1B1R1K1 w kq - 0 1").unwrap();
    b.iter(|| pos.pseudolegal_moves().collect::<Vec<_>>());
}

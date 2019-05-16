// Copyright 2017-2019 Sean Gillespie.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

#[macro_use]
extern crate criterion;

use apollo3::attacks;
use apollo3::{Bitboard, MoveGenerator, MoveVec, Position, Square};
use criterion::black_box;
use criterion::Criterion;

fn queen_attacks(square: Square, occ: Bitboard) -> Bitboard {
    attacks::queen_attacks(square, occ)
}

fn knight_attacks(square: Square) -> Bitboard {
    attacks::knight_attacks(square)
}

fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("queen attacks f5 empty board", |b| {
        b.iter(|| queen_attacks(black_box(Square::F5), Bitboard::none()))
    });

    c.bench_function("knight attacks f5", |b| {
        b.iter(|| knight_attacks(black_box(Square::F5)))
    });

    c.bench_function("position clone", |b| {
        let pos = Position::from_start_position();
        b.iter(|| black_box(&pos).clone())
    });

    c.bench_function("generate moves start", |b| {
        let pos = Position::from_start_position();
        b.iter(|| {
            let mut vec = MoveVec::default();
            let gen = MoveGenerator::new();
            gen.generate_moves(black_box(&pos), &mut vec);
        });
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);

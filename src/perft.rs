// Copyright 2017-2019 Sean Gillespie.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.
use rayon::prelude::*;

use crate::move_generator::{MoveGenerator, MoveVec};
use crate::position::Position;

pub fn perft(pos: &Position, depth: u32, use_legality_test: bool) -> u64 {
    if depth == 0 {
        return 1;
    }

    let mut moves = MoveVec::default();
    let movegen = MoveGenerator::new();
    movegen.generate_moves(pos, &mut moves);
    return moves
        .par_iter()
        .map(|&mov| {
            if use_legality_test {
                if pos.is_legal_given_pseudolegal(mov) {
                    let mut new_pos = pos.clone();
                    //let side_to_move = pos.side_to_move();
                    new_pos.apply_move(mov);
                    /*
                    if new_pos.is_check(side_to_move) {
                        println!("{}", pos);
                        println!("move {} is not legal, but legality check said yes", mov);
                        panic!();
                    }
                    */
                    perft(&new_pos, depth - 1, use_legality_test)
                } else {
                    /*
                    let mut new_pos = pos.clone();
                    let side_to_move = pos.side_to_move();
                    new_pos.apply_move(mov);
                    if !new_pos.is_check(side_to_move) {
                        println!("{}", pos);
                        println!("move {} is legal, but legality check said no", mov);
                        panic!();
                    } else {
                        0
                    }
                    */
                    0
                }
            } else {
                let mut new_pos = pos.clone();
                let side_to_move = pos.side_to_move();
                new_pos.apply_move(mov);
                if !new_pos.is_check(side_to_move) {
                    perft(&new_pos, depth - 1, use_legality_test)
                } else {
                    0
                }
            }
        })
        .sum();
}

#[cfg(test)]
mod tests {
    use super::perft;
    use crate::position::Position;

    fn perft_test(fen: &'static str, depth: u32, count: u64) {
        let pos = Position::from_fen(fen).unwrap();
        for &legality_test in &[false, true] {
            let res = perft(&pos, depth, legality_test);
            assert_eq!(res, count);
        }
    }

    macro_rules! perft_tests {
        () => {};
        ($name:ident ($depth:expr): $fen:expr => $count:expr; $($tail:tt)*) => {
            #[test]
            fn $name() {
                perft_test($fen, $depth, $count)
            }

            perft_tests!($($tail)*);
        };

        (skip $name:ident ($depth:expr): $fen:expr => $count:expr; $($tail:tt)*) => {
            #[test]
            #[ignore]
            fn $name() {
                perft_test($fen, $depth, $count)
            }

            perft_tests!($($tail)*);
        };

    }

    perft_tests! {
        start_1 (1): "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1" => 20;
        start_2 (2): "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1" => 400;
        start_3 (3): "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1" => 8902;
        start_4 (4): "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1" => 197281;

        kiwipete_1 (1): "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1" => 48;
        kiwipete_2 (2): "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1" => 2039;
        kiwipete_3 (3): "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1" => 97862;

        position_3_1 (1): "8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - - 0 1" => 14;
        position_3_2 (2): "8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - - 0 1" => 191;
        position_3_3 (3): "8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - - 0 1" => 2812;
        position_3_4 (4): "8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - - 0 1" => 43238;

        position_4_1 (1): "r2q1rk1/pP1p2pp/Q4n2/bbp1p3/Np6/1B3NBn/pPPP1PPP/R3K2R b KQ - 0 1" => 6;
        position_4_2 (2): "r2q1rk1/pP1p2pp/Q4n2/bbp1p3/Np6/1B3NBn/pPPP1PPP/R3K2R b KQ - 0 1" => 264;
        position_4_3 (3): "r2q1rk1/pP1p2pp/Q4n2/bbp1p3/Np6/1B3NBn/pPPP1PPP/R3K2R b KQ - 0 1" => 9467;
        position_4_4 (4): "r2q1rk1/pP1p2pp/Q4n2/bbp1p3/Np6/1B3NBn/pPPP1PPP/R3K2R b KQ - 0 1" => 422333;

        position_5_1 (1): "rnbq1k1r/pp1Pbppp/2p5/8/2B5/8/PPP1NnPP/RNBQK2R w KQ - 1 8" => 44;
        position_5_2 (2): "rnbq1k1r/pp1Pbppp/2p5/8/2B5/8/PPP1NnPP/RNBQK2R w KQ - 1 8" => 1486;
        position_5_3 (3): "rnbq1k1r/pp1Pbppp/2p5/8/2B5/8/PPP1NnPP/RNBQK2R w KQ - 1 8" => 62379;
        position_5_4 (4): "rnbq1k1r/pp1Pbppp/2p5/8/2B5/8/PPP1NnPP/RNBQK2R w KQ - 1 8" => 2103487;
    }
}

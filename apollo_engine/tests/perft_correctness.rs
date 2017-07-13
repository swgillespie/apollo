// Copyright 2017 Sean Gillespie.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.
extern crate apollo_engine;
use apollo_engine::Position;

#[derive(Clone, Debug, Default)]
pub struct PerftResults {
    pub nodes: u64,
    pub captures: u64,
    pub en_passants: u32,
    pub castles: u32,
    pub promotions: u32,
    pub checks: u32,
    pub checkmates: u32,
}

pub fn perft(fen: &str, depth: u32) -> PerftResults {
    let mut results = Default::default();
    let position = Position::from_fen(fen).unwrap();
    perft_impl(position, depth, &mut results);
    results
}

fn perft_impl(pos: Position, depth: u32, results: &mut PerftResults) {
    if depth == 0 {
        results.nodes += 1;
        return;
    }

    let mut seen_legal_move = false;
    let moves: Vec<_> = pos.pseudolegal_moves().collect();
    for _m in moves.clone() {
        //println!("generated move: {}", _m);
    }

    for mov in moves {
        let mut new_pos = pos.clone();
        let to_move = new_pos.side_to_move();
        new_pos.apply_move(mov);
        if !new_pos.is_check(to_move) {
            seen_legal_move = true;
            if mov.is_capture() {
                results.captures += 1;
            }

            if mov.is_en_passant() {
                results.en_passants += 1;
            }

            if mov.is_kingside_castle() || mov.is_queenside_castle() {
                results.castles += 1;
            }

            if mov.is_promotion() {
                results.promotions += 1;
            }

            if new_pos.is_check(to_move.toggle()) {
                results.checks += 1;
            }

            perft_impl(new_pos, depth - 1, results);
        }
    }

    if !seen_legal_move {
        results.checkmates += 1;
    }
}

mod initial_position {
    use super::*;

    #[test]
    fn perft_1() {
        apollo_engine::initialize();
        let results = perft("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1",
                            1);
        assert_eq!(20, results.nodes);
        assert_eq!(0, results.captures);
        assert_eq!(0, results.en_passants);
        assert_eq!(0, results.castles);
        assert_eq!(0, results.promotions);
        assert_eq!(0, results.checks);
        assert_eq!(0, results.checkmates);
    }

    #[test]
    fn perft_2() {
        apollo_engine::initialize();
        let results = perft("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1",
                            2);
        assert_eq!(400, results.nodes);
        assert_eq!(0, results.captures);
        assert_eq!(0, results.en_passants);
        assert_eq!(0, results.castles);
        assert_eq!(0, results.promotions);
        assert_eq!(0, results.checks);
        assert_eq!(0, results.checkmates);
    }

    #[test]
    fn perft_3() {
        apollo_engine::initialize();
        let results = perft("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1",
                            3);
        assert_eq!(8902, results.nodes);
        assert_eq!(34, results.captures);
        assert_eq!(0, results.en_passants);
        assert_eq!(0, results.castles);
        assert_eq!(0, results.promotions);
        assert_eq!(12, results.checks);
        assert_eq!(0, results.checkmates);
    }

    #[test]
    fn perft_4() {
        apollo_engine::initialize();
        let results = perft("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1",
                            4);
        assert_eq!(197281, results.nodes);
        //assert_eq!(1576, results.captures);
        //assert_eq!(0, results.en_passants);
        //assert_eq!(0, results.castles);
        //assert_eq!(0, results.promotions);
        //assert_eq!(469, results.checks);
        //assert_eq!(8, results.checkmates);
    }

    #[test]
    #[ignore]
    fn perft_5() {
        apollo_engine::initialize();
        let results = perft("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1",
                            5);
        assert_eq!(4865609, results.nodes);
        //assert_eq!(82719, results.captures);
        //assert_eq!(258, results.en_passants);
        //assert_eq!(0, results.castles);
        //assert_eq!(0, results.promotions);
        //assert_eq!(27351, results.checks);
        //assert_eq!(347, results.checkmates);
    }
}

mod kiwipete {
    use super::*;

    #[test]
    fn perft_1() {
        apollo_engine::initialize();
        let results = perft("r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1",
                            1);
        assert_eq!(48, results.nodes);
        //assert_eq!(8, results.captures);
        //assert_eq!(0, results.en_passants);
        //assert_eq!(2, results.castles);
        //assert_eq!(0, results.promotions);
        //assert_eq!(0, results.checks);
        //assert_eq!(0, results.checkmates);
    }

    #[test]
    fn perft_2() {
        apollo_engine::initialize();
        let results = perft("r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1",
                            2);
        assert_eq!(2039, results.nodes);
        //assert_eq!(351, results.captures);
        //assert_eq!(1, results.en_passants);
        //assert_eq!(91, results.castles);
        //assert_eq!(0, results.promotions);
        //assert_eq!(3, results.checks);
        //assert_eq!(0, results.checkmates);
    }

    #[test]
    fn perft_3() {
        apollo_engine::initialize();
        let results = perft("r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1",
                            3);
        assert_eq!(97862, results.nodes);
        //assert_eq!(17102, results.captures);
        //assert_eq!(45, results.en_passants);
        //assert_eq!(3162, results.castles);
        //assert_eq!(0, results.promotions);
        //assert_eq!(993, results.checks);
        //assert_eq!(1, results.checkmates);
    }
}

mod position_4 {
    use super::*;

    #[test]
    fn perft_1() {
        apollo_engine::initialize();
        let results = perft("r3k2r/Pppp1ppp/1b3nbN/nP6/BBP1P3/q4N2/Pp1P2PP/R2Q1RK1 w kq - 0 1", 1);
        assert_eq!(6, results.nodes);
    }

    #[test]
    fn perft_2() {
        apollo_engine::initialize();
        let results = perft("r3k2r/Pppp1ppp/1b3nbN/nP6/BBP1P3/q4N2/Pp1P2PP/R2Q1RK1 w kq - 0 1", 2);
        assert_eq!(264, results.nodes);
    }

    #[test]
    fn perft_3() {
        apollo_engine::initialize();
        let results = perft("r3k2r/Pppp1ppp/1b3nbN/nP6/BBP1P3/q4N2/Pp1P2PP/R2Q1RK1 w kq - 0 1", 3);
        assert_eq!(9467, results.nodes);
    }

    #[test]
    fn perft_4() {
        apollo_engine::initialize();
        let results = perft("r3k2r/Pppp1ppp/1b3nbN/nP6/BBP1P3/q4N2/Pp1P2PP/R2Q1RK1 w kq - 0 1", 4);
        assert_eq!(422333, results.nodes);
    }

    #[test]
    #[ignore]
    fn perft_5() {
        apollo_engine::initialize();
        let results = perft("r3k2r/Pppp1ppp/1b3nbN/nP6/BBP1P3/q4N2/Pp1P2PP/R2Q1RK1 w kq - 0 1", 5);
        assert_eq!(15833292, results.nodes);
    }
}

mod position_5 {
    use super::*;

    #[test]
    fn perft_1() {
        apollo_engine::initialize();
        let results = perft("rnbq1k1r/pp1Pbppp/2p5/8/2B5/8/PPP1NnPP/RNBQK2R w KQ - 1 8",
                            1);
        assert_eq!(44, results.nodes);
    }

    #[test]
    fn perft_2() {
        apollo_engine::initialize();
        let results = perft("rnbq1k1r/pp1Pbppp/2p5/8/2B5/8/PPP1NnPP/RNBQK2R w KQ - 1 8",
                            2);
        assert_eq!(1486, results.nodes);
    }

    #[test]
    fn perft_3() {
        apollo_engine::initialize();
        let results = perft("rnbq1k1r/pp1Pbppp/2p5/8/2B5/8/PPP1NnPP/RNBQK2R w KQ - 1 8",
                            3);
        assert_eq!(62379, results.nodes);
    }

    #[test]
    fn perft_4() {
        apollo_engine::initialize();
        let results = perft("rnbq1k1r/pp1Pbppp/2p5/8/2B5/8/PPP1NnPP/RNBQK2R w KQ - 1 8",
                            4);
        assert_eq!(2103487, results.nodes);
    }

    #[test]
    #[ignore]
    fn perft_5() {
        apollo_engine::initialize();
        let results = perft("rnbq1k1r/pp1Pbppp/2p5/8/2B5/8/PPP1NnPP/RNBQK2R w KQ - 1 8",
                            5);
        assert_eq!(89941194, results.nodes);
    }
}
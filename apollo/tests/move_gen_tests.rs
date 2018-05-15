// Copyright 2017 Sean Gillespie.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.
extern crate apollo;
use std::collections::HashSet;
use apollo::{Move, Position, Square, PieceKind};

fn assert_moves_generated(fen: &'static str, moves: &[Move]) {
    apollo::initialize();
    let pos = Position::from_fen(fen).unwrap();
    let moves_hash : HashSet<_> = pos.pseudolegal_moves().collect();
    for mov in moves_hash {
        if !moves.contains(&mov) {
            println!("move {} was not found in collection: ", mov);
            for m in moves {
                println!("   > {}", m);
            }

            panic!()
        }
    }
}

fn assert_moves_contains(fen: &'static str, moves: &[Move]) {
    apollo::initialize();
    let pos = Position::from_fen(fen).unwrap();
    let moves_hash : HashSet<_> = pos.pseudolegal_moves().collect();
    for mov in moves {
        if !moves_hash.contains(mov) {
            println!("move {} was not generated", mov);
            panic!()
        }
    }
}

fn assert_moves_does_not_contain(fen: &'static str, moves: &[Move]) {
    apollo::initialize();
    let pos = Position::from_fen(fen).unwrap();
    let moves_hash : HashSet<_> = pos.pseudolegal_moves().collect();
    for mov in moves {
        if moves_hash.contains(mov) {
            println!("move list contained banned move: {}", mov);
            panic!()
        }
    }
}

mod pawns {
    use super::*;

    #[test]
    fn white_pawn_smoke_test() {
        assert_moves_generated("8/8/8/8/5P2/8/8/8 w - - 0 1", &[
            Move::quiet(Square::F4, Square::F5)
        ]);
    }

    #[test]
    fn white_pawn_starting_rank() {
        assert_moves_generated("8/8/8/8/8/8/4P3/8 w - - 0 1", &[
            Move::quiet(Square::E2, Square::E3),
            Move::double_pawn_push(Square::E2, Square::E4)
        ]);
    }

    #[test]
    fn white_pawn_en_passant() {
        assert_moves_generated("8/8/4PpP1/8/8/8/8/8 w - f7 0 1", &[
            Move::quiet(Square::E6, Square::E7),
            Move::quiet(Square::G6, Square::G7),
            Move::en_passant(Square::G6, Square::F7),
            Move::en_passant(Square::E6, Square::F7)
        ]);
    }

    #[test]
    fn white_pawn_promotion() {
        assert_moves_generated("8/4P3/8/8/8/8/8/8 w - - 0 1", &[
            Move::promotion(Square::E7, Square::E8, PieceKind::Knight),
            Move::promotion(Square::E7, Square::E8, PieceKind::Bishop),
            Move::promotion(Square::E7, Square::E8, PieceKind::Rook),
            Move::promotion(Square::E7, Square::E8, PieceKind::Queen)
        ]);
    }

    #[test]
    fn white_pawn_promo_capture() {
        assert_moves_generated("5b2/4P3/8/8/8/8/8/8 w - - 0 1", &[
            Move::promotion(Square::E7, Square::E8, PieceKind::Knight),
            Move::promotion(Square::E7, Square::E8, PieceKind::Bishop),
            Move::promotion(Square::E7, Square::E8, PieceKind::Rook),
            Move::promotion(Square::E7, Square::E8, PieceKind::Queen),
            Move::promotion_capture(Square::E7, Square::F8, PieceKind::Knight),
            Move::promotion_capture(Square::E7, Square::F8, PieceKind::Bishop),
            Move::promotion_capture(Square::E7, Square::F8, PieceKind::Rook),
            Move::promotion_capture(Square::E7, Square::F8, PieceKind::Queen)
        ]);
    }

    #[test]
    fn no_pawn_push_when_target_square_occupied() {
        assert_moves_does_not_contain("rnbqkbnr/1ppppppp/8/p7/P7/8/1PPPPPPP/RNBQKBNR w KQkq - 0 1", &[
            Move::quiet(Square::A4, Square::A5)
        ]);
    }

    #[test]
    fn no_double_pawn_push_when_blocked() {
        assert_moves_does_not_contain("8/8/8/8/8/4p3/4P3/8 w - - 0 1", &[
            Move::double_pawn_push(Square::E2, Square::E4)
        ]);
    }

    #[test]
    fn kiwipete_bug_1() {
        assert_moves_contains("r3k2r/p1ppqpb1/bn2pnp1/3PN3/Pp2P3/2N2Q1p/1PPBBPPP/R3K2R b KQkq a3 0 1", &[
            Move::en_passant(Square::B4, Square::A3)
        ]);
    }

    #[test]
    fn illegal_en_passant() {
        assert_moves_does_not_contain("8/8/4p3/8/8/8/5P2/8 w - e7 0 1", &[
            // this can happen if we are sloppy about validating the legality
            // of EP-moves
            Move::en_passant(Square::F2, Square::E7)
        ]);
    }
}

mod bishops {
    use super::*;

    #[test]
    fn smoke_test() {
        assert_moves_generated("8/8/8/8/3B4/8/8/8 w - - 0 1", &[
            Move::quiet(Square::D4, Square::E5),
            Move::quiet(Square::D4, Square::F6),
            Move::quiet(Square::D4, Square::G7),
            Move::quiet(Square::D4, Square::H8),
            Move::quiet(Square::D4, Square::E3),
            Move::quiet(Square::D4, Square::F2),
            Move::quiet(Square::D4, Square::G1),
            Move::quiet(Square::D4, Square::C3),
            Move::quiet(Square::D4, Square::B2),
            Move::quiet(Square::D4, Square::A1),
            Move::quiet(Square::D4, Square::C5),
            Move::quiet(Square::D4, Square::B6),
            Move::quiet(Square::D4, Square::A7)
        ]);
    }

    #[test]
    fn smoke_capture() {
        assert_moves_generated("8/8/8/2p1p3/3B4/2p1p3/8/8 w - - 0 1", &[
            Move::capture(Square::D4, Square::E5),
            Move::capture(Square::D4, Square::E3),
            Move::capture(Square::D4, Square::C5),
            Move::capture(Square::D4, Square::C3)
        ]);
    }
}

mod kings {
    use super::*;

    #[test]
    fn smoke_test() {
        assert_moves_generated("8/8/8/8/4K3/8/8/8 w - - 0 1", &[
            Move::quiet(Square::E4, Square::E5),
            Move::quiet(Square::E4, Square::F5),
            Move::quiet(Square::E4, Square::F4),
            Move::quiet(Square::E4, Square::F3),
            Move::quiet(Square::E4, Square::E3),
            Move::quiet(Square::E4, Square::D3),
            Move::quiet(Square::E4, Square::D4),
            Move::quiet(Square::E4, Square::D5)
        ]);
    }
    
    #[test]
    fn kingside_castle() {
        assert_moves_contains("8/8/8/8/8/8/8/4K2R w K - 0 1", &[
            Move::kingside_castle(Square::E1, Square::G1)
        ]);
    }

    #[test]
    fn queenside_castle() {
        assert_moves_contains("8/8/8/8/8/8/8/R3K3 w Q - 0 1", &[
            Move::queenside_castle(Square::E1, Square::C1)
        ]);
    }

    #[test]
    fn kingside_castle_neg() {
        assert_moves_does_not_contain("8/8/8/8/8/8/8/4K2R w Q - 0 1", &[
            Move::kingside_castle(Square::E1, Square::G1)
        ]);
    }

    #[test]
    fn queenside_castle_neg() {
        assert_moves_does_not_contain("8/8/8/8/8/8/8/R3K3 w K - 0 1", &[
            Move::queenside_castle(Square::E1, Square::C1)
        ]);
    }

    #[test]
    fn castle_through_check() {
        assert_moves_does_not_contain("8/8/8/8/5r2/8/8/4K2R w - - 0 1", &[
            Move::kingside_castle(Square::E1, Square::G1)
        ]);
    }

    #[test]
    fn kingside_castle_when_space_occupied() {
        assert_moves_does_not_contain("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1", &[
            Move::kingside_castle(Square::E1, Square::G1)
        ]);
    }

    #[test]
    fn queenside_castle_when_space_occupied() {
        assert_moves_does_not_contain("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1", &[
            Move::queenside_castle(Square::E1, Square::C1)
        ]);
    }

    #[test]
    fn kiwipete_bug_2() {
        assert_moves_contains("r3k2r/p1pNqpb1/bn2pnp1/3P4/1p2P3/2N2Q1p/PPPBBPPP/R3K2R b KQkq - 0 1", &[
            Move::queenside_castle(Square::E8, Square::C8)
        ]);
    }

    #[test]
    fn kiwipete_bug_3() {
        assert_moves_does_not_contain("2kr3r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/5Q1p/PPPBBPPP/RN2K2R w KQ - 2 2", &[
            // there's a knight on b1, this blocks castling even though it
            // doesn't block the king's movement
            Move::queenside_castle(Square::E1, Square::C1)
        ])
    }
}

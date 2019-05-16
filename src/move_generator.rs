// Copyright 2017-2019 Sean Gillespie.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.
use arrayvec::ArrayVec;

use crate::attacks;
use crate::bitboard::Bitboard;
use crate::moves::Move;
use crate::position::Position;
use crate::types::{Color, Direction, PieceKind, Rank, Square};

/// MoveVec is the type of array vectors that are large enough to contain all of the possible moves
/// that are pseudolegal from a given chess position. MoveVec is small enough to be allocated on the
/// stack so that move generation can proceed with zero allocations.
pub type MoveVec = ArrayVec<[Move; 224]>;

/// MoveGenerator is an iterator for chess moves that are pseudolegal from a given position.
/// Generating legal moves from a board is significantly more expensive than generating pseudolegal
/// ones, so this move generator is designed to be as fast as possible while still generating
/// moves that are mostly legal.
///
/// Moves that are known to be pseudolegal (i.e. generated by this iterator) but not legal:
///    1. Moves of pieces that are absolutely pinned
///    2. Moves of the king into check
///    3. Moves that leave the king in check
///
/// Basically, any move that may result in check is often pseudolegal. Full legality can be tested
/// efficiently on a move-by-move basis given pseudolegality, so that check is often done before
/// applying the move.
///
/// Note that, despite presenting an iterator-like interface, MoveGenerator generates all moves
/// eagerly. Later improvements may make the move generator a little more lazy and, ideally,
/// configurable depending on what sort of move search the searcher wants to perform. Many chess
/// engines have specialized move generators for quiescence searches or searching while in check
/// that can be accomodated by this MoveGenerator.
pub struct MoveGenerator;

impl MoveGenerator {
    pub fn new() -> MoveGenerator {
        MoveGenerator
    }

    pub fn generate_moves(&self, pos: &Position, buf: &mut MoveVec) {
        self.generate_pawn_moves(pos, buf);
        self.generate_knight_moves(pos, buf);
        self.generate_sliding_moves(pos, buf, |c| pos.bishops(c), attacks::bishop_attacks);
        self.generate_sliding_moves(pos, buf, |c| pos.rooks(c), attacks::rook_attacks);
        self.generate_sliding_moves(pos, buf, |c| pos.queens(c), attacks::queen_attacks);
        self.generate_king_moves(pos, buf);
    }

    fn generate_pawn_moves(&self, pos: &Position, buf: &mut MoveVec) {
        let color = pos.side_to_move();
        let enemy_pieces = pos.pieces(color.toggle());
        let allied_pieces = pos.pieces(color);
        let pieces = enemy_pieces.or(allied_pieces);
        let (start_rank, promo_rank, pawn_dir, ep_dir) = match color {
            Color::White => (Rank::Two, Rank::Eight, Direction::North, Direction::South),
            Color::Black => (Rank::Seven, Rank::One, Direction::South, Direction::North),
        };

        for pawn in pos.pawns(color) {
            // Pawns shouldn't be on the promotion rank.
            assert!(
                pawn.rank() != promo_rank,
                "no pawns should be on the promotion rank"
            );

            let target = pawn.towards(pawn_dir);

            // Non-capturing moves.
            if !pieces.test(target) {
                if target.rank() == promo_rank {
                    buf.push(Move::promotion(pawn, target, PieceKind::Knight));
                    buf.push(Move::promotion(pawn, target, PieceKind::Bishop));
                    buf.push(Move::promotion(pawn, target, PieceKind::Rook));
                    buf.push(Move::promotion(pawn, target, PieceKind::Queen));
                } else {
                    buf.push(Move::quiet(pawn, target));
                }
            }

            // Double pawn pushes, for pawns originating on the starting rank.
            if pawn.rank() == start_rank {
                let two_push_target = target.towards(pawn_dir);
                if !pieces.test(target) && !pieces.test(two_push_target) {
                    buf.push(Move::double_pawn_push(pawn, two_push_target));
                }
            }

            // Non-en-passant capturing moves.
            for target in attacks::pawn_attacks(pawn, color) {
                if enemy_pieces.test(target) {
                    assert!(
                        !allied_pieces.test(target),
                        "square can't be occupied by both allied and enemy pieces"
                    );
                    if target.rank() == promo_rank {
                        buf.push(Move::promotion_capture(pawn, target, PieceKind::Knight));
                        buf.push(Move::promotion_capture(pawn, target, PieceKind::Bishop));
                        buf.push(Move::promotion_capture(pawn, target, PieceKind::Rook));
                        buf.push(Move::promotion_capture(pawn, target, PieceKind::Queen));
                    } else {
                        buf.push(Move::capture(pawn, target));
                    }
                }
            }

            // En-passant moves.
            if let Some(ep_square) = pos.en_passant_square() {
                // Would this move be a normal legal attack for this pawn?
                if attacks::pawn_attacks(pawn, color).test(ep_square) {
                    // If so, the attack square is directly behind the pawn that was pushed.
                    let attack_square = ep_square.towards(ep_dir);
                    assert!(
                        enemy_pieces.test(attack_square),
                        "square behind EP-square should be occupied"
                    );
                    assert!(!pieces.test(ep_square), "EP-square should be unoccupied");
                    buf.push(Move::en_passant(pawn, ep_square));
                }
            }
        }
    }

    fn generate_knight_moves(&self, pos: &Position, buf: &mut MoveVec) {
        let color = pos.side_to_move();
        let enemy_pieces = pos.pieces(color.toggle());
        let allied_pieces = pos.pieces(color);
        for knight in pos.knights(color) {
            for target in attacks::knight_attacks(knight) {
                if enemy_pieces.test(target) {
                    buf.push(Move::capture(knight, target));
                } else if !allied_pieces.test(target) {
                    buf.push(Move::quiet(knight, target));
                }
            }
        }
    }

    fn generate_sliding_moves<B, A>(&self, pos: &Position, buf: &mut MoveVec, board: B, attacks: A)
    where
        B: Fn(Color) -> Bitboard,
        A: Fn(Square, Bitboard) -> Bitboard,
    {
        let color = pos.side_to_move();
        let enemy_pieces = pos.pieces(color.toggle());
        let allied_pieces = pos.pieces(color);
        let pieces = enemy_pieces.or(allied_pieces);
        for piece in board(color) {
            for target in attacks(piece, pieces) {
                // In theory we only need to test the end of rays for occupancy, but this works.
                if enemy_pieces.test(target) {
                    buf.push(Move::capture(piece, target));
                } else if !allied_pieces.test(target) {
                    buf.push(Move::quiet(piece, target));
                }
            }
        }
    }

    fn generate_king_moves(&self, pos: &Position, buf: &mut MoveVec) {
        let color = pos.side_to_move();
        let enemy_pieces = pos.pieces(color.toggle());
        let allied_pieces = pos.pieces(color);
        let pieces = enemy_pieces.or(allied_pieces);
        for king in pos.kings(color) {
            for target in attacks::king_attacks(king) {
                if enemy_pieces.test(target) {
                    buf.push(Move::capture(king, target));
                } else if !allied_pieces.test(target) {
                    buf.push(Move::quiet(king, target));
                }
            }

            // Generate castling moves, if we are allowed to castle.
            if pos.is_check(color) {
                // No castling out of check.
                continue;
            }

            if pos.can_castle_kingside(color) {
                let starting_rook = if color == Color::White {
                    Square::H1
                } else {
                    Square::H8
                };

                if let Some(piece) = pos.piece_at(starting_rook) {
                    if piece.kind == PieceKind::Rook && piece.color == color {
                        let one = king.towards(Direction::East);
                        let two = one.towards(Direction::East);
                        if !pieces.test(one) && !pieces.test(two) {
                            // The king moves across both squares one and two and it is illegal
                            // to castle through check. We can only proceed if no enemy piece is
                            // attacking the squares the king travels upon.
                            if pos.squares_attacking(color.toggle(), one).empty()
                                && pos.squares_attacking(color.toggle(), two).empty()
                            {
                                buf.push(Move::kingside_castle(king, two));
                            }
                        }
                    }
                }
            }

            if pos.can_castle_queenside(color) {
                let starting_rook = if color == Color::White {
                    Square::A1
                } else {
                    Square::A8
                };

                if let Some(piece) = pos.piece_at(starting_rook) {
                    if piece.kind == PieceKind::Rook && piece.color == color {
                        let one = king.towards(Direction::West);
                        let two = one.towards(Direction::West);
                        let three = two.towards(Direction::West);
                        if !pieces.test(one) && !pieces.test(two) && !pieces.test(three) {
                            // Square three can be checked, but it can't be occupied. The rook
                            // travels across square three, but the king does not.
                            if pos.squares_attacking(color.toggle(), one).empty()
                                && pos.squares_attacking(color.toggle(), two).empty()
                            {
                                buf.push(Move::queenside_castle(king, two));
                            }
                        }
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use super::{MoveGenerator, MoveVec};
    use crate::moves::Move;
    use crate::position::Position;
    use crate::types::{PieceKind, Square};

    fn assert_moves_generated(fen: &'static str, moves: &[Move]) {
        let pos = Position::from_fen(fen).unwrap();
        let gen = MoveGenerator::new();
        let mut mov_vec = MoveVec::default();
        gen.generate_moves(&pos, &mut mov_vec);
        let hash: HashSet<_> = mov_vec.iter().collect();
        for mov in hash {
            if !moves.contains(&mov) {
                println!("move {} was not found in collection: ", mov);
                for m in moves {
                    println!("   > {}", m);
                }

                println!("{}", pos);
                panic!()
            }
        }
    }

    fn assert_moves_contains(fen: &'static str, moves: &[Move]) {
        let pos = Position::from_fen(fen).unwrap();
        let gen = MoveGenerator::new();
        let mut mov_vec = MoveVec::default();
        gen.generate_moves(&pos, &mut mov_vec);
        let hash: HashSet<_> = mov_vec.iter().collect();
        for mov in moves {
            if !hash.contains(mov) {
                println!("move {} was not generated", mov);
                println!("{}", pos);
                panic!()
            }
        }
    }

    fn assert_moves_does_not_contain(fen: &'static str, moves: &[Move]) {
        let pos = Position::from_fen(fen).unwrap();
        let gen = MoveGenerator::new();
        let mut mov_vec = MoveVec::default();
        gen.generate_moves(&pos, &mut mov_vec);
        let hash: HashSet<_> = mov_vec.iter().collect();
        for mov in moves {
            if hash.contains(mov) {
                println!("move list contained banned move: {}", mov);
                println!("{}", pos);
                panic!()
            }
        }
    }

    mod pawns {
        use super::*;

        #[test]
        fn white_pawn_smoke_test() {
            assert_moves_generated(
                "8/8/8/8/5P2/8/8/8 w - - 0 1",
                &[Move::quiet(Square::F4, Square::F5)],
            );
        }

        #[test]
        fn white_pawn_starting_rank() {
            assert_moves_generated(
                "8/8/8/8/8/8/4P3/8 w - - 0 1",
                &[
                    Move::quiet(Square::E2, Square::E3),
                    Move::double_pawn_push(Square::E2, Square::E4),
                ],
            );
        }

        #[test]
        fn white_pawn_en_passant() {
            assert_moves_generated(
                "8/8/4PpP1/8/8/8/8/8 w - f7 0 1",
                &[
                    Move::quiet(Square::E6, Square::E7),
                    Move::quiet(Square::G6, Square::G7),
                    Move::en_passant(Square::G6, Square::F7),
                    Move::en_passant(Square::E6, Square::F7),
                ],
            );
        }

        #[test]
        fn white_pawn_promotion() {
            assert_moves_generated(
                "8/4P3/8/8/8/8/8/8 w - - 0 1",
                &[
                    Move::promotion(Square::E7, Square::E8, PieceKind::Knight),
                    Move::promotion(Square::E7, Square::E8, PieceKind::Bishop),
                    Move::promotion(Square::E7, Square::E8, PieceKind::Rook),
                    Move::promotion(Square::E7, Square::E8, PieceKind::Queen),
                ],
            );
        }

        #[test]
        fn white_pawn_promo_capture() {
            assert_moves_generated(
                "5b2/4P3/8/8/8/8/8/8 w - - 0 1",
                &[
                    Move::promotion(Square::E7, Square::E8, PieceKind::Knight),
                    Move::promotion(Square::E7, Square::E8, PieceKind::Bishop),
                    Move::promotion(Square::E7, Square::E8, PieceKind::Rook),
                    Move::promotion(Square::E7, Square::E8, PieceKind::Queen),
                    Move::promotion_capture(Square::E7, Square::F8, PieceKind::Knight),
                    Move::promotion_capture(Square::E7, Square::F8, PieceKind::Bishop),
                    Move::promotion_capture(Square::E7, Square::F8, PieceKind::Rook),
                    Move::promotion_capture(Square::E7, Square::F8, PieceKind::Queen),
                ],
            );
        }

        #[test]
        fn no_pawn_push_when_target_square_occupied() {
            assert_moves_does_not_contain(
                "rnbqkbnr/1ppppppp/8/p7/P7/8/1PPPPPPP/RNBQKBNR w KQkq - 0 1",
                &[Move::quiet(Square::A4, Square::A5)],
            );
        }

        #[test]
        fn no_double_pawn_push_when_blocked() {
            assert_moves_does_not_contain(
                "8/8/8/8/8/4p3/4P3/8 w - - 0 1",
                &[Move::double_pawn_push(Square::E2, Square::E4)],
            );
        }

        #[test]
        fn kiwipete_bug_1() {
            assert_moves_contains(
                "r3k2r/p1ppqpb1/bn2pnp1/3PN3/Pp2P3/2N2Q1p/1PPBBPPP/R3K2R b KQkq a3 0 1",
                &[Move::en_passant(Square::B4, Square::A3)],
            );
        }

        #[test]
        fn illegal_en_passant() {
            assert_moves_does_not_contain(
                "8/8/4p3/8/8/8/5P2/8 w - e7 0 1",
                &[
                    // this can happen if we are sloppy about validating the legality
                    // of EP-moves
                    Move::en_passant(Square::F2, Square::E7),
                ],
            );
        }
    }

    mod bishops {
        use super::*;

        #[test]
        fn smoke_test() {
            assert_moves_generated(
                "8/8/8/8/3B4/8/8/8 w - - 0 1",
                &[
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
                    Move::quiet(Square::D4, Square::A7),
                ],
            );
        }

        #[test]
        fn smoke_capture() {
            assert_moves_generated(
                "8/8/8/2p1p3/3B4/2p1p3/8/8 w - - 0 1",
                &[
                    Move::capture(Square::D4, Square::E5),
                    Move::capture(Square::D4, Square::E3),
                    Move::capture(Square::D4, Square::C5),
                    Move::capture(Square::D4, Square::C3),
                ],
            );
        }
    }

    mod kings {
        use super::*;

        #[test]
        fn smoke_test() {
            assert_moves_generated(
                "8/8/8/8/4K3/8/8/8 w - - 0 1",
                &[
                    Move::quiet(Square::E4, Square::E5),
                    Move::quiet(Square::E4, Square::F5),
                    Move::quiet(Square::E4, Square::F4),
                    Move::quiet(Square::E4, Square::F3),
                    Move::quiet(Square::E4, Square::E3),
                    Move::quiet(Square::E4, Square::D3),
                    Move::quiet(Square::E4, Square::D4),
                    Move::quiet(Square::E4, Square::D5),
                ],
            );
        }

        #[test]
        fn position_4_check_block() {
            assert_moves_contains(
                "r2q1rk1/pP1p2pp/Q4n2/bbp1p3/Np6/1B3NBn/pPPP1PPP/R3K2R b KQ - 0 1",
                &[
                    Move::quiet(Square::C5, Square::C4),
                    Move::double_pawn_push(Square::D7, Square::D5),
                    Move::quiet(Square::B5, Square::C4),
                    Move::quiet(Square::F6, Square::D5),
                    Move::quiet(Square::F8, Square::F7),
                    Move::quiet(Square::G8, Square::H8),
                ],
            );
        }

        #[test]
        fn kingside_castle() {
            assert_moves_contains(
                "8/8/8/8/8/8/8/4K2R w K - 0 1",
                &[Move::kingside_castle(Square::E1, Square::G1)],
            );
        }

        #[test]
        fn queenside_castle() {
            assert_moves_contains(
                "8/8/8/8/8/8/8/R3K3 w Q - 0 1",
                &[Move::queenside_castle(Square::E1, Square::C1)],
            );
        }

        #[test]
        fn kingside_castle_neg() {
            assert_moves_does_not_contain(
                "8/8/8/8/8/8/8/4K2R w Q - 0 1",
                &[Move::kingside_castle(Square::E1, Square::G1)],
            );
        }

        #[test]
        fn queenside_castle_neg() {
            assert_moves_does_not_contain(
                "8/8/8/8/8/8/8/R3K3 w K - 0 1",
                &[Move::queenside_castle(Square::E1, Square::C1)],
            );
        }

        #[test]
        fn castle_through_check() {
            assert_moves_does_not_contain(
                "8/8/8/8/5r2/8/8/4K2R w - - 0 1",
                &[Move::kingside_castle(Square::E1, Square::G1)],
            );
        }

        #[test]
        fn kingside_castle_when_space_occupied() {
            assert_moves_does_not_contain(
                "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1",
                &[Move::kingside_castle(Square::E1, Square::G1)],
            );
        }

        #[test]
        fn queenside_castle_when_space_occupied() {
            assert_moves_does_not_contain(
                "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1",
                &[Move::queenside_castle(Square::E1, Square::C1)],
            );
        }

        #[test]
        fn kiwipete_bug_2() {
            assert_moves_contains(
                "r3k2r/p1pNqpb1/bn2pnp1/3P4/1p2P3/2N2Q1p/PPPBBPPP/R3K2R b KQkq - 0 1",
                &[Move::queenside_castle(Square::E8, Square::C8)],
            );
        }

        #[test]
        fn kiwipete_bug_3() {
            assert_moves_does_not_contain(
                "2kr3r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/5Q1p/PPPBBPPP/RN2K2R w KQ - 2 2",
                &[
                    // there's a knight on b1, this blocks castling even though it
                    // doesn't block the king's movement
                    Move::queenside_castle(Square::E1, Square::C1),
                ],
            )
        }
    }
}
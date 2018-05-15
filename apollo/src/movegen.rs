// Copyright 2017 Sean Gillespie.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.
use position::Position;
use moves::Move;
use types::{Color, Direction, Rank, PieceKind, Square};
use bitboard::Bitboard;
use attacks;

fn add_pawns(pos: &Position, moves: &mut Vec<Move>) {
    let color = pos.side_to_move();
    let enemy_piece_map = pos.color(color.toggle());
    let allied_piece_map = pos.color(color);
    let piece_map = allied_piece_map | enemy_piece_map;
    let (starting_rank, promo_rank, pawn_direction, ep_dir) = match color {
        Color::White => (Rank::Rank2, Rank::Rank8, Direction::North, Direction::South),
        Color::Black => (Rank::Rank7, Rank::Rank1, Direction::South, Direction::North),
    };

    for pawn in pos.pawns(color) {
        // pawns shouldn't be on the promotion rank
        assert!(pawn.rank() != promo_rank);

        let target = pawn.towards(pawn_direction)
            .expect("moving a pawn forward one should always stay on board");

        // non-capturing moves
        if target.rank() == promo_rank && !piece_map.test(target) {
            moves.push(Move::promotion(pawn, target, PieceKind::Bishop));
            moves.push(Move::promotion(pawn, target, PieceKind::Knight));
            moves.push(Move::promotion(pawn, target, PieceKind::Rook));
            moves.push(Move::promotion(pawn, target, PieceKind::Queen));
        } else if !piece_map.test(target) {
            moves.push(Move::quiet(pawn, target));
        }

        // double-pawn pushes, for pawns still on their starting square
        if pawn.rank() == starting_rank {
            let two_push_target = target.towards(pawn_direction)
                .expect("double-push from starting rank should stay on board");
            if !piece_map.test(two_push_target) && !piece_map.test(target) {
                moves.push(Move::double_pawn_push(pawn, two_push_target));
            }
        }

        // non-ep capturing moves
        for attack_sq in pos.engine().attack_table().pawn_attacks(pawn, color) {
            if enemy_piece_map.test(attack_sq) {
                assert!(!allied_piece_map.test(attack_sq));
                if attack_sq.rank() == promo_rank {
                    moves.push(Move::promotion_capture(pawn, attack_sq, PieceKind::Bishop));
                    moves.push(Move::promotion_capture(pawn, attack_sq, PieceKind::Knight));
                    moves.push(Move::promotion_capture(pawn, attack_sq, PieceKind::Rook));
                    moves.push(Move::promotion_capture(pawn, attack_sq, PieceKind::Queen));
                } else {
                    moves.push(Move::capture(pawn, attack_sq));
                }
            }
        }

        // en-passant
        if let Some(ep_square) = pos.en_passant_square() {
            // would this be a normal legal attack for this pawn?
            if pos.engine().attack_table().pawn_attacks(pawn, color).test(ep_square) {
                // the attack square is directly behind the pawn that was pushed
                let attack_sq =
                    ep_square.towards(ep_dir).expect("en-passant piece square not on board");
                assert!(enemy_piece_map.test(attack_sq));
                assert!(!piece_map.test(ep_square));
                moves.push(Move::en_passant(pawn, ep_square));
            }
        }
    }
}

fn add_knights(pos: &Position, moves: &mut Vec<Move>) {
    let color = pos.side_to_move();
    let enemy_piece_map = pos.color(color.toggle());
    let allied_piece_map = pos.color(color);
    for knight in pos.knights(color) {
        for atk in pos.engine().attack_table().knight_attacks(knight) {
            if enemy_piece_map.test(atk) {
                moves.push(Move::capture(knight, atk));
            } else if !allied_piece_map.test(atk) {
                moves.push(Move::quiet(knight, atk));
            }
        }
    }
}

fn add_sliding_pieces<F, B>(pos: &Position, moves: &mut Vec<Move>, atks: F, board: B)
    where F: Fn(Square, Bitboard) -> Bitboard,
          B: FnOnce(Color) -> Bitboard
{
    let color = pos.side_to_move();
    let enemy_piece_map = pos.color(color.toggle());
    let allied_piece_map = pos.color(color);
    for piece in board(color) {
        for atk in atks(piece, enemy_piece_map | allied_piece_map) {
            // in theory we only need to test the end of rays
            // for occupancy...
            if enemy_piece_map.test(atk) {
                moves.push(Move::capture(piece, atk));
            } else if !allied_piece_map.test(atk) {
                moves.push(Move::quiet(piece, atk));
            }
        }
    }
}

fn add_kings(pos: &Position, moves: &mut Vec<Move>) {
    let color = pos.side_to_move();
    let enemy_piece_map = pos.color(color.toggle());
    let allied_piece_map = pos.color(color);
    let piece_map = enemy_piece_map | allied_piece_map;
    assert!(pos.kings(color).count() <= 1);
    if let Some(king) = pos.kings(color).first() {
        for atk in pos.engine().attack_table().king_attacks(king) {
            if enemy_piece_map.test(atk) {
                moves.push(Move::capture(king, atk));
            } else if !allied_piece_map.test(atk) {
                moves.push(Move::quiet(king, atk));
            }
        }

        if pos.is_check(color) {
            // can't castle out of check
            return;
        }

        if pos.can_castle_kingside(color) {
            let one = king.towards(Direction::East).unwrap();
            let two = one.towards(Direction::East).unwrap();
            if !piece_map.test(one) && !piece_map.test(two) {
                if pos.squares_attacking(color.toggle(), one).empty() &&
                   pos.squares_attacking(color.toggle(), two).empty() {
                    moves.push(Move::kingside_castle(king, two));
                }
            }
        }

        if pos.can_castle_queenside(color) {
            let one = king.towards(Direction::West).unwrap();
            let two = one.towards(Direction::West).unwrap();
            let three = two.towards(Direction::West).unwrap();
            // three can be checked, but it can't be occupied. this is because
            // the rook needs to move "across" three, but the king does not.
            if !piece_map.test(one) && !piece_map.test(two) && !piece_map.test(three) {
                if pos.squares_attacking(color.toggle(), one).empty() &&
                   pos.squares_attacking(color.toggle(), two).empty() {
                    moves.push(Move::queenside_castle(king, two));
                }
            }
        }
    }
}

pub fn generate_moves(pos: &Position, moves: &mut Vec<Move>) {
    add_pawns(pos, moves);
    add_knights(pos, moves);
    add_sliding_pieces(pos,
                       moves,
                       |s, b| pos.engine().attack_table().bishop_attacks(s, b),
                       |c| pos.bishops(c));
    add_sliding_pieces(pos,
                       moves,
                       |s, b| pos.engine().attack_table().rook_attacks(s, b),
                       |c| pos.rooks(c));
    add_sliding_pieces(pos,
                       moves,
                       |s, b| pos.engine().attack_table().queen_attacks(s, b),
                       |c| pos.queens(c));
    add_kings(pos, moves);
}

// Copyright 2017 Sean Gillespie.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.
extern crate apollo_engine;
use apollo_engine::{Position, Move, Square, Color, PieceKind};

#[test]
fn smoke_test_opening_pawn() {
    let mut pos = Position::from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 2 1").unwrap();

    // nothing fancy, move a pawn up one.
    pos.apply_move(Move::quiet(Square::E2, Square::E3));

    // it should now be Black's turn to move.
    assert_eq!(Color::Black, pos.side_to_move());

    // the fullmove clock shouldn't have incremented
    // (it only increments every Black move)
    assert_eq!(1, pos.fullmove_clock());

    // a pawn moved, so the halfmove clock should be zero.
    assert_eq!(0, pos.halfmove_clock());

    // there should be a pawn on e3
    let pawn = pos.piece_at(Square::E3).unwrap();
    assert_eq!(PieceKind::Pawn, pawn.kind);
    assert_eq!(Color::White, pawn.color);

    // there should not be a pawn on e2
    let not_pawn = pos.piece_at(Square::E2);
    assert!(not_pawn.is_none());
}

#[test]
fn en_passant_reset() {
    // EP square at e3, black to move
    let mut pos = Position::from_fen("8/8/8/8/4Pp2/8/8/8 b - e3 0 1").unwrap();

    // black not taking EP opportunity
    pos.apply_move(Move::quiet(Square::F4, Square::F3));

    // EP no longer possible.
    assert_eq!(Color::White, pos.side_to_move());
    assert_eq!(None, pos.en_passant_square());
}

#[test]
fn double_pawn_push_sets_ep() {
    // white to move
    let mut pos = Position::from_fen("8/8/8/8/8/8/4P3/8 w - - 0 1").unwrap();

    // white double-pawn pushes
    pos.apply_move(Move::double_pawn_push(Square::E2, Square::E4));

    // now black to move, with EP square set
    assert_eq!(Color::Black, pos.side_to_move());
    assert_eq!(Some(Square::E3), pos.en_passant_square());
}

#[test]
fn basic_capture() {
    let mut pos = Position::from_fen("8/8/8/8/5p2/4P3/8/8 w - - 2 1").unwrap();
    pos.apply_move(Move::capture(Square::E3, Square::F4));

    // There should be a white pawn on F4
    let piece = pos.piece_at(Square::F4).unwrap();
    assert_eq!(PieceKind::Pawn, piece.kind);
    assert_eq!(Color::White, piece.color);

    // There should be no piece on E3
    let other_piece = pos.piece_at(Square::E3);
    assert!(other_piece.is_none());

    // The halfmove clock should reset (capture)
    assert_eq!(0, pos.halfmove_clock());
}

#[test]
fn non_pawn_quiet_move() {
    let mut pos = Position::from_fen("8/8/8/8/8/8/4B3/8 w - - 5 2").unwrap();
    pos.apply_move(Move::quiet(Square::E2, Square::G4));

    // the halfmove clock should not be reset.
    assert_eq!(6, pos.halfmove_clock());
}

#[test]
fn moving_king_castle_status() {
   let mut pos = Position::from_fen("8/8/8/8/8/8/8/4K2R w KQ - 0 1").unwrap();

   // white's turn to move, white moves its king.
   pos.apply_move(Move::quiet(Square::E1, Square::E2));

   // white can't castle anymore.
   assert!(!pos.can_castle_kingside(Color::White));
   assert!(!pos.can_castle_queenside(Color::White));
}

#[test]
fn moving_kingside_rook_castle_status() {
    let mut pos = Position::from_fen("8/8/8/8/8/8/8/4K2R w KQ - 0 1").unwrap();

    // white's turn to move, white moves its kingside rook.
    pos.apply_move(Move::quiet(Square::H1, Square::G1));

    // white can't castle kingside anymore
    assert!(!pos.can_castle_kingside(Color::White));
    assert!(pos.can_castle_queenside(Color::White));
}

#[test]
fn moving_queenside_rook_castle_status() {
    let mut pos = Position::from_fen("8/8/8/8/8/8/8/R3K3 w KQ - 0 1").unwrap();

    // white's turn to move, white moves its queenside rook.
    pos.apply_move(Move::quiet(Square::A1, Square::B1));

    // white can't castle queenside anymore
    assert!(!pos.can_castle_queenside(Color::White));
    assert!(pos.can_castle_kingside(Color::White));
}

#[test]
fn rook_capture_castle_status() {
    // tests that we can't capture if there's no rook on the target
    // square, even if the rooks themselves never moved (i.e. they
    // were captured on their starting square)
    let mut pos = Position::from_fen("8/8/8/8/8/7r/4P3/R3K2R b KQ - 0 1").unwrap();

    // black to move, black captures the rook at H1
    pos.apply_move(Move::capture(Square::H3, Square::H1));

    // white to move, white pushes the pawn
    pos.apply_move(Move::double_pawn_push(Square::E2, Square::E4));

    // black to move, black moves the rook
    pos.apply_move(Move::quiet(Square::H1, Square::H5));

    // white moves the queenside rook to the kingside rook
    // start location
    pos.apply_move(Move::quiet(Square::A1, Square::A2));
    pos.apply_move(Move::quiet(Square::H5, Square::H6));
    pos.apply_move(Move::quiet(Square::A2, Square::H2));
    pos.apply_move(Move::quiet(Square::H6, Square::H7));
    pos.apply_move(Move::quiet(Square::H2, Square::H1));

    // white shouldn't be able to castle kingside, despite
    // there being a rook on the kingside rook square
    // and us never moving the kingside rook
    assert!(!pos.can_castle_kingside(Color::White));
}

#[test]
fn en_passant_capture() {
    // tests that we remove an ep-captured piece from its
    // actual location and not try to remove the EP-square
    let mut pos = Position::from_fen("8/8/8/3pP3/8/8/8/8 w - d6 0 1").unwrap();

    // white to move, white EP-captures the pawn
    pos.apply_move(Move::en_passant(Square::E5, Square::D6));

    // there should not be a piece at D5 anymore
    let black_pawn = pos.piece_at(Square::D5);
    assert!(black_pawn.is_none());

    // the white pawn should be at the EP-square
    let white_pawn = pos.piece_at(Square::D6).unwrap();
    assert_eq!(Color::White, white_pawn.color);
    assert_eq!(PieceKind::Pawn, white_pawn.kind);
}

#[test]
fn basic_promotion() {
    let mut pos = Position::from_fen("8/4P3/8/8/8/8/8/8 w - - 0 1").unwrap();

    // white to move, white promotes the pawn on e7
    pos.apply_move(Move::promotion(Square::E7, Square::E8, PieceKind::Queen));

    // there should be a queen on e8
    let queen = pos.piece_at(Square::E8).unwrap();
    assert_eq!(Color::White, queen.color);
    assert_eq!(PieceKind::Queen, queen.kind);
}

#[test]
fn basic_promote_capture() {
    let mut pos = Position::from_fen("5b2/4P3/8/8/8/8/8/8 w - - 0 1").unwrap();

    // white to move, white promote-captures the pawn on e7 and captures
    // the bishop
    pos.apply_move(Move::promotion_capture(Square::E7, Square::F8, PieceKind::Queen));

    // there should be a white queen on f8
    let queen = pos.piece_at(Square::F8).unwrap();
    assert_eq!(Color::White, queen.color);
    assert_eq!(PieceKind::Queen, queen.kind);
}
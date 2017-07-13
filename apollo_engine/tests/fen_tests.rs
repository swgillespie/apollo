// Copyright 2017 Sean Gillespie.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.
extern crate apollo_engine;
extern crate num_traits; 

use apollo_engine::{Position, FenParseError};
use apollo_engine::{Square, Color, File, Rank, Piece, PieceKind};
use num_traits::FromPrimitive;

#[test]
fn fen_smoke() {
    let pos = Position::from_fen("8/8/8/8/8/8/8/8 w - - 0 0").unwrap();

    // white's turn to move.
    assert_eq!(Color::White, pos.side_to_move());

    // no castling.
    assert!(!pos.can_castle_kingside(Color::White));
    assert!(!pos.can_castle_kingside(Color::Black));
    assert!(!pos.can_castle_queenside(Color::White));
    assert!(!pos.can_castle_queenside(Color::Black));

    // no en passant.
    assert!(pos.en_passant_square().is_none());

    // both clocks are zero.
    assert_eq!(0, pos.halfmove_clock());
    assert_eq!(0, pos.fullmove_clock());
}

#[test]
fn starting_position() {
    let pos = Position::from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1")
        .unwrap();

    let check_square = |square: &'static str, piece: Piece| {
        assert!(square.len() == 2);
        let chars : Vec<_> = square.chars().collect();
        let file = File::from_char(chars[0]).unwrap();
        let rank = Rank::from_char(chars[1]).unwrap();
        let square = Square::of(rank, file);
        let piece_on_square = pos.piece_at(square).unwrap();
        assert_eq!(piece.kind, piece_on_square.kind);
        assert_eq!(piece.color, piece_on_square.color);
    };

    let check_vacant = |square: Square| {
        assert!(pos.piece_at(square).is_none());
    };

    check_square("a1", Piece::new(PieceKind::Rook, Color::White));
    check_square("b1", Piece::new(PieceKind::Knight, Color::White));
    check_square("c1", Piece::new(PieceKind::Bishop, Color::White));
    check_square("d1", Piece::new(PieceKind::Queen, Color::White));
    check_square("e1", Piece::new(PieceKind::King, Color::White));
    check_square("f1", Piece::new(PieceKind::Bishop, Color::White));
    check_square("g1", Piece::new(PieceKind::Knight, Color::White));
    check_square("h1", Piece::new(PieceKind::Rook, Color::White));
    check_square("a2", Piece::new(PieceKind::Pawn, Color::White));
    check_square("b2", Piece::new(PieceKind::Pawn, Color::White));
    check_square("c2", Piece::new(PieceKind::Pawn, Color::White));
    check_square("d2", Piece::new(PieceKind::Pawn, Color::White));
    check_square("e2", Piece::new(PieceKind::Pawn, Color::White));
    check_square("f2", Piece::new(PieceKind::Pawn, Color::White));
    check_square("g2", Piece::new(PieceKind::Pawn, Color::White));
    check_square("h2", Piece::new(PieceKind::Pawn, Color::White));

    for sq in (Square::A3 as usize)..(Square::A7 as usize) {
        let sq_actual = FromPrimitive::from_u64(sq as u64).unwrap();
        check_vacant(sq_actual);
    }

    check_square("a7", Piece::new(PieceKind::Pawn, Color::Black));
    check_square("b7", Piece::new(PieceKind::Pawn, Color::Black));
    check_square("c7", Piece::new(PieceKind::Pawn, Color::Black));
    check_square("d7", Piece::new(PieceKind::Pawn, Color::Black));
    check_square("e7", Piece::new(PieceKind::Pawn, Color::Black));
    check_square("f7", Piece::new(PieceKind::Pawn, Color::Black));
    check_square("g7", Piece::new(PieceKind::Pawn, Color::Black));
    check_square("h7", Piece::new(PieceKind::Pawn, Color::Black));
    check_square("a8", Piece::new(PieceKind::Rook, Color::Black));
    check_square("b8", Piece::new(PieceKind::Knight, Color::Black));
    check_square("c8", Piece::new(PieceKind::Bishop, Color::Black));
    check_square("d8", Piece::new(PieceKind::Queen, Color::Black));
    check_square("e8", Piece::new(PieceKind::King, Color::Black));
    check_square("f8", Piece::new(PieceKind::Bishop, Color::Black));
    check_square("g8", Piece::new(PieceKind::Knight, Color::Black));
    check_square("h8", Piece::new(PieceKind::Rook, Color::Black));

    assert!(pos.can_castle_kingside(Color::White));
    assert!(pos.can_castle_kingside(Color::Black));
    assert!(pos.can_castle_queenside(Color::White));
    assert!(pos.can_castle_queenside(Color::Black));
}

#[test]
fn empty() {
    let err = Position::from_fen("").unwrap_err();
    assert_eq!(FenParseError::UnexpectedEnd, err);
}

#[test]
fn unknown_piece() {
    let err = Position::from_fen("z7/8/8/8/8/8/8/8 w - - 0 0").unwrap_err();
    assert_eq!(FenParseError::UnknownPiece, err);
}

#[test]
fn invalid_digit() {
    let err = Position::from_fen("9/8/8/8/8/8/8/8 w - - 0 0").unwrap_err();
    assert_eq!(FenParseError::InvalidDigit, err);
}

#[test]
fn not_sum_to_8() {
    let err = Position::from_fen("pppp5/8/8/8/8/8/8/8 w - - 0 0").unwrap_err();
    assert_eq!(FenParseError::FileDoesNotSumToEight, err);
}

#[test]
fn bad_side_to_move() {
    let err = Position::from_fen("8/8/8/8/8/8/8/8 c - - 0 0").unwrap_err();
    assert_eq!(FenParseError::InvalidSideToMove, err);
}

#[test]
fn bad_castle_status() {
    let err = Position::from_fen("8/8/8/8/8/8/8/8 w a - 0 0").unwrap_err();
    assert_eq!(FenParseError::InvalidCastle, err);
}

#[test]
fn bad_en_passant() {
    let err = Position::from_fen("8/8/8/8/8/8/8/8 w - 88 0 0").unwrap_err();
    assert_eq!(FenParseError::InvalidEnPassant, err);
}

#[test]
fn empty_halfmove() {
    let err = Position::from_fen("8/8/8/8/8/8/8/8 w - - q 0").unwrap_err();
    assert_eq!(FenParseError::EmptyHalfmove, err);
}

#[test]
fn invalid_halfmove() {
    let err = Position::from_fen("8/8/8/8/8/8/8/8 w - - 4294967296 0").unwrap_err();
    assert_eq!(FenParseError::InvalidHalfmove, err);
}

#[test]
fn empty_fullmove() {
    let err = Position::from_fen("8/8/8/8/8/8/8/8 w - - 0 q").unwrap_err();
    assert_eq!(FenParseError::EmptyFullmove, err);
}

#[test]
fn fullmove_early_end() {
    let err = Position::from_fen("8/8/8/8/8/8/8/8 w - - 0").unwrap_err();
    assert_eq!(FenParseError::UnexpectedEnd, err);
}


#[test]
fn invalid_fullmove() {
    let err = Position::from_fen("8/8/8/8/8/8/8/8 w - - 0 4294967296").unwrap_err();
    assert_eq!(FenParseError::InvalidFullmove, err);
}

fn fen_roundtrip(fen: &'static str) {
    let pos = Position::from_fen(fen).unwrap();
    assert_eq!(fen, pos.as_fen());
}

#[test]
fn starting_position_roundtrip() {
    fen_roundtrip("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1");
}

#[test]
fn empty_roundtrip() {
    fen_roundtrip("8/8/8/8/8/8/8/8 w - - 0 1");
}
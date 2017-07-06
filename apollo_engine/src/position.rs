// Copyright 2017 Sean Gillespie. See the COPYRIGHT
// file at the top-level directory of this distribution.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.
use num_traits::FromPrimitive;
use bitboard::Bitboard;
use types::{self, Square, Piece, Color, CastleStatus};

pub enum FenParseError {

}

pub struct Position {
    boards_by_piece: [Bitboard; 12],
    boards_by_color: [Bitboard; 2],
    en_passant_square: Option<Square>,
    halfmove_clock: u32,
    fullmove_clock: u32,
    side_to_move: Color,
    castle_status: CastleStatus,
}

impl Position {
    /// Constructs a new, empty Position.
    pub fn new() -> Position {
        Position {
            boards_by_piece: [Bitboard::none(); 12],
            boards_by_color: [Bitboard::none(); 2],
            en_passant_square: None,
            halfmove_clock: 0,
            fullmove_clock: 0,
            side_to_move: Color::White,
            castle_status: types::CASTLE_ALL,
        }
    }

    /// Constructs a new position from a FEN representation of a board
    /// position.
    pub fn from_fen<S: AsRef<str>>(_fen: S) -> Result<Position, FenParseError> {
        unimplemented!()
    }

    /// Adds a piece to the board at the given square, returning `Ok` if
    /// the adding was successful (i.e. the space was unoccupied) and `Err`
    /// if the space was occupied.
    pub fn add_piece(&mut self, square: Square, piece: Piece) -> Result<(), ()> {
        if self.piece_at(square).is_some() {
            return Err(());
        }

        self.boards_by_color[piece.color as usize].set(square);
        let offset = if piece.color == Color::White { 0 } else { 6 };
        self.boards_by_piece[piece.kind as usize + offset].set(square);
        Ok(())
    }

    /// Removes a piece from the board at the given square, returning `Ok` if
    /// the removal was successful (i.e. the space was occupied) and `Err`
    /// if the space was empty.
    pub fn remove_piece(&mut self, square: Square) -> Result<(), ()> {
        if let Some(piece) = self.piece_at(square) {
            self.boards_by_color[piece.color as usize].unset(square);
            let offset = if piece.color == Color::White { 0 } else { 6 };
            self.boards_by_piece[piece.kind as usize + offset].unset(square);
            Ok(())
        } else {
            Err(())
        }
    }

    /// Finds a piece located at a given square, returning it if one exists.
    pub fn piece_at(&self, square: Square) -> Option<Piece> {
        let (board_offset, color) = if self.boards_by_color[Color::White as usize].test(square) {
            (0, Color::White)
        } else if self.boards_by_color[Color::Black as usize]
                      .test(square) {
            (6, Color::Black)
        } else {
            return None;
        };

        for piece in 0...5 {
            // pawn through king
            let board = self.boards_by_piece[piece + board_offset];
            if board.test(square) {
                let kind = FromPrimitive::from_u64(piece as u64).unwrap();
                return Some(Piece::new(kind, color));
            }
        }

        // if we get here, we failed to update a bitboard somewhere.
        unreachable!()
    }

    /// Returns the current en-passant square for this position, if
    /// there is one.
    pub fn en_passant_square(&self) -> Option<Square> {
        self.en_passant_square
    }

    /// Returns the current halfmove clock for this position.
    pub fn halfmove_clock(&self) -> u32 {
        self.halfmove_clock
    }

    /// Returns the current fullmove clock for this position.
    pub fn fullmove_clock(&self) -> u32 {
        self.fullmove_clock
    }

    /// Returns the current side to move for this position.
    pub fn side_to_move(&self) -> Color {
        self.side_to_move
    }

    /// Returns whether or not the given color is permitted to castle
    /// kingside in this position.
    pub fn can_castle_kingside(&self, color: Color) -> bool {
        match color {
            Color::White => self.castle_status.contains(types::WHITE_O_O),
            Color::Black => self.castle_status.contains(types::BLACK_O_O),
        }
    }

    /// Returns whether or not the given color is permitted to castle queenside
    /// in this position.
    pub fn can_castle_queenside(&self, color: Color) -> bool {
        match color {
            Color::White => self.castle_status.contains(types::WHITE_O_O_O),
            Color::Black => self.castle_status.contains(types::BLACK_O_O_O),
        }
    }
}
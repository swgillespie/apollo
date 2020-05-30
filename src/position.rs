// Copyright 2017-2019 Sean Gillespie.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.
use std::convert::TryFrom;
use std::fmt::{self, Write};

use crate::attacks;
use crate::bitboard::Bitboard;
use crate::bitboard::{BB_RANK_1, BB_RANK_2, BB_RANK_7, BB_RANK_8};
use crate::move_generator::{MoveGenerator, MoveVec};
use crate::moves::Move;
use crate::types::TableIndex;
use crate::types::{CastleStatus, Color, Direction, File, Piece, PieceKind, Rank, Square};
use crate::types::{FILES, PIECE_KINDS, RANKS};
use crate::zobrist;

/// Possible errors that can arise when parsing a FEN string into a `Position`.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum FenParseError {
    UnexpectedChar(char),
    UnexpectedEnd,
    InvalidDigit,
    FileDoesNotSumToEight,
    UnknownPiece,
    InvalidSideToMove,
    InvalidCastle,
    InvalidEnPassant,
    EmptyHalfmove,
    InvalidHalfmove,
    EmptyFullmove,
    InvalidFullmove,
}

#[derive(Clone, Debug)]
pub struct Position {
    boards_by_piece: [Bitboard; 12],
    boards_by_color: [Bitboard; 2],
    en_passant_square: Option<Square>,
    halfmove_clock: u32,
    fullmove_clock: u32,
    side_to_move: Color,
    castle_status: CastleStatus,
    zobrist_hash: u64,
}

//
// Board state getters
//

impl Position {
    pub const fn new() -> Position {
        Position {
            boards_by_piece: [Bitboard::none(); 12],
            boards_by_color: [Bitboard::none(); 2],
            en_passant_square: None,
            halfmove_clock: 0,
            fullmove_clock: 0,
            side_to_move: Color::White,
            castle_status: CastleStatus::NONE,
            zobrist_hash: 0,
        }
    }

    pub fn en_passant_square(&self) -> Option<Square> {
        self.en_passant_square
    }

    pub fn halfmove_clock(&self) -> u32 {
        self.halfmove_clock
    }

    pub fn fullmove_clock(&self) -> u32 {
        self.fullmove_clock
    }

    pub fn side_to_move(&self) -> Color {
        self.side_to_move
    }

    pub fn zobrist_hash(&self) -> u64 {
        self.zobrist_hash
    }

    pub fn can_castle_kingside(&self, color: Color) -> bool {
        match color {
            Color::White => self.castle_status.contains(CastleStatus::WHITE_KINGSIDE),
            Color::Black => self.castle_status.contains(CastleStatus::BLACK_KINGSIDE),
        }
    }

    pub fn can_castle_queenside(&self, color: Color) -> bool {
        match color {
            Color::White => self.castle_status.contains(CastleStatus::WHITE_QUEENSIDE),
            Color::Black => self.castle_status.contains(CastleStatus::BLACK_QUEENSIDE),
        }
    }

    pub fn pieces(&self, color: Color) -> Bitboard {
        self.boards_by_color[color.as_index()]
    }

    pub fn pieces_of_kind(&self, color: Color, kind: PieceKind) -> Bitboard {
        let offset = match color {
            Color::White => 0,
            Color::Black => 6,
        };
        self.boards_by_piece[offset + kind.as_index()]
    }

    pub fn pawns(&self, color: Color) -> Bitboard {
        self.pieces_of_kind(color, PieceKind::Pawn)
    }

    pub fn bishops(&self, color: Color) -> Bitboard {
        self.pieces_of_kind(color, PieceKind::Bishop)
    }

    pub fn knights(&self, color: Color) -> Bitboard {
        self.pieces_of_kind(color, PieceKind::Knight)
    }

    pub fn rooks(&self, color: Color) -> Bitboard {
        self.pieces_of_kind(color, PieceKind::Rook)
    }

    pub fn queens(&self, color: Color) -> Bitboard {
        self.pieces_of_kind(color, PieceKind::Queen)
    }

    pub fn kings(&self, color: Color) -> Bitboard {
        self.pieces_of_kind(color, PieceKind::King)
    }
}

//
// Move application and board manipulation
//

impl Position {
    pub fn add_piece(&mut self, square: Square, piece: Piece) -> Result<(), ()> {
        if self.piece_at(square).is_some() {
            return Err(());
        }

        self.boards_by_color[piece.color as usize].set(square);
        let offset = if piece.color == Color::White { 0 } else { 6 };
        self.boards_by_piece[piece.kind as usize + offset].set(square);
        zobrist::modify_piece(&mut self.zobrist_hash, square, piece);
        Ok(())
    }

    pub fn remove_piece(&mut self, square: Square) -> Result<(), ()> {
        let existing_piece = if let Some(piece) = self.piece_at(square) {
            piece
        } else {
            return Err(());
        };

        self.boards_by_color[existing_piece.color.as_index()].unset(square);
        let offset = if existing_piece.color == Color::White {
            0
        } else {
            6
        };
        self.boards_by_piece[existing_piece.kind.as_index() + offset].unset(square);
        zobrist::modify_piece(&mut self.zobrist_hash, square, existing_piece);
        Ok(())
    }

    pub fn piece_at(&self, square: Square) -> Option<Piece> {
        let (board_offset, color) = if self.boards_by_color[Color::White as usize].test(square) {
            (0, Color::White)
        } else if self.boards_by_color[Color::Black as usize].test(square) {
            (6, Color::Black)
        } else {
            return None;
        };

        for &kind in &PIECE_KINDS {
            let board = self.boards_by_piece[kind as usize + board_offset];
            if board.test(square) {
                return Some(Piece::new(kind, color));
            }
        }

        // If we get here, we failed to update a bitboard somewhere.
        unreachable!()
    }

    pub fn apply_move(&mut self, mov: Move) {
        // Quick out for null moves:
        //  1. EP is not legal next turn.
        //  2. Halfmove clock always increases.
        //  3. Fullmove clock increases if Black makes the null move.
        if mov.is_null() {
            self.en_passant_square = None;
            self.side_to_move = self.side_to_move.toggle();
            zobrist::modify_side_to_move(&mut self.zobrist_hash);
            if self.side_to_move == Color::White {
                self.fullmove_clock += 1;
            }
            return;
        }

        let moving_piece = self
            .piece_at(mov.source())
            .expect("invalid move: no piece at source square");

        // If this move is a capture, we need to remove the captured piece from the board before we
        // proceed.
        if mov.is_capture() {
            // The target square is often the destination square of the move, except in the case of
            // en-passant where the target square lies on an adjacent file.
            let target_square = if !mov.is_en_passant() {
                mov.destination()
            } else {
                // En-passant moves are the only case when the piece being captured does
                // not lie on the same square as the move destination.
                let ep_dir = if self.side_to_move == Color::White {
                    Direction::South
                } else {
                    Direction::North
                };

                let ep_square = self
                    .en_passant_square
                    .expect("invalid move: EP without EP-square");
                ep_square.towards(ep_dir)
            };

            // Remove the piece from the board - it has been captured.
            self.remove_piece(target_square)
                .expect("invalid move: no piece at capture target");

            // If this piece is a rook on its starting square, invalidate the castle for the other
            // player.
            if target_square == kingside_rook(self.side_to_move.toggle()) {
                self.castle_status &= !kingside_castle_mask(self.side_to_move.toggle());
                zobrist::modify_kingside_castle(&mut self.zobrist_hash, self.side_to_move.toggle());
            } else if target_square == queenside_rook(self.side_to_move.toggle()) {
                self.castle_status &= !queenside_castle_mask(self.side_to_move.toggle());
                zobrist::modify_queenside_castle(
                    &mut self.zobrist_hash,
                    self.side_to_move.toggle(),
                );
            }
        }

        // The move destination square is now guaranteed to be empty. Next we need to handle moves
        // that end up in places other than the destination square.
        if mov.is_castle() {
            // Castles are encoded using the king's start and stop position. Notably, the rook is
            // not at the move's destination square.
            //
            // Castles are also interesting in that two pieces move, so we'll handle the move of
            // the rook here and handle the movement of the king later on in the function.
            let (post_castle_dir, pre_castle_dir, num_squares) = if mov.is_kingside_castle() {
                (Direction::West, Direction::East, 1)
            } else {
                (Direction::East, Direction::West, 2)
            };

            let new_rook_square = mov.destination().towards(post_castle_dir);
            let mut rook_square = mov.destination();
            for _ in 0..num_squares {
                rook_square = rook_square.towards(pre_castle_dir);
            }

            let rook = self
                .piece_at(rook_square)
                .expect("invalid move: castle without rook");
            self.remove_piece(rook_square).unwrap();
            self.add_piece(new_rook_square, rook)
                .expect("invalid move: piece at rook target square");
        }

        // Now, we're going to add the moving piece to the destination square. Unless this is a
        // promotion, the piece that we add to the destination is the piece that is currently at
        // the source square.
        let piece_to_add = if mov.is_promotion() {
            Piece::new(mov.promotion_piece(), self.side_to_move)
        } else {
            moving_piece
        };

        self.remove_piece(mov.source())
            .expect("invalid move: no piece at source square");
        self.add_piece(mov.destination(), piece_to_add)
            .expect("invalid move: piece at destination square");
        if mov.is_double_pawn_push() {
            // Double pawn pushes set the en-passant square.
            let ep_dir = if self.side_to_move == Color::White {
                Direction::South
            } else {
                Direction::North
            };

            let ep_square = mov.destination().towards(ep_dir);
            zobrist::modify_en_passant(
                &mut self.zobrist_hash,
                self.en_passant_square,
                Some(ep_square),
            );
            self.en_passant_square = Some(ep_square);
        } else {
            // All other moves clear the en-passant square.
            zobrist::modify_en_passant(&mut self.zobrist_hash, self.en_passant_square, None);
            self.en_passant_square = None;
        }

        // Re-calculate our castle status. Side to move may have invalidated their castle rights
        // by moving their rooks or king.
        if moving_piece.kind == PieceKind::Rook {
            // Moving a rook invalidates the castle on that rook's side of the board.

            if self.can_castle_queenside(self.side_to_move)
                && mov.source() == queenside_rook(self.side_to_move)
            {
                // Move of the queenside rook. Can't castle queenside anymore.
                self.castle_status &= !queenside_castle_mask(self.side_to_move);
                zobrist::modify_queenside_castle(&mut self.zobrist_hash, self.side_to_move);
            } else if self.can_castle_kingside(self.side_to_move)
                && mov.source() == kingside_rook(self.side_to_move)
            {
                // Move of the kingside rook. Can't castle kingside anymore.
                self.castle_status &= !kingside_castle_mask(self.side_to_move);
                zobrist::modify_kingside_castle(&mut self.zobrist_hash, self.side_to_move);
            }
        } else if moving_piece.kind == PieceKind::King {
            // Moving a king invalides the castle on both sides of the board.
            self.castle_status &= !castle_mask(self.side_to_move);
            zobrist::modify_queenside_castle(&mut self.zobrist_hash, self.side_to_move);
            zobrist::modify_kingside_castle(&mut self.zobrist_hash, self.side_to_move);
        }

        self.side_to_move = self.side_to_move.toggle();
        zobrist::modify_side_to_move(&mut self.zobrist_hash);
        if mov.is_capture() || moving_piece.kind == PieceKind::Pawn {
            self.halfmove_clock = 0;
        } else {
            self.halfmove_clock += 1;
        }

        if self.side_to_move == Color::White {
            self.fullmove_clock += 1;
        }
    }
}

//
// Board analysis (check detection, pin detection, etc.)
//

impl Position {
    pub fn squares_attacking(&self, to_move: Color, target: Square) -> Bitboard {
        let mut attacks = Bitboard::none();

        // Pretend that there's a "super-piece" at the target square and see if it hits anything.
        // This covers all pieces except for kings and pawns.
        let occupancy = self.pieces(Color::White) | self.pieces(Color::Black);

        // Queen attacks cover bishops, rooks, and queens, so check that first.
        let sliding_pieces = self.pieces_of_kind(to_move, PieceKind::Queen)
            | self.pieces_of_kind(to_move, PieceKind::Rook)
            | self.pieces_of_kind(to_move, PieceKind::Bishop);
        let sliding_attacks = attacks::queen_attacks(target, occupancy).and(sliding_pieces);
        if !sliding_attacks.empty() {
            // Hit - there's something that might be attacking via a slide. However, since we're
            // modeling a superpiece, we need to check that the attacking pieces actually can legally
            // attack this square.
            for attacker in sliding_attacks {
                let piece = self
                    .piece_at(attacker)
                    .expect("attack table produced piece not on board?");
                if piece.attacks(attacker, occupancy).test(target) {
                    attacks.set(attacker);
                }
            }
        }

        // Knight attacks are straightforward since knight moves are symmetric.
        let knight_attacks = attacks::knight_attacks(target).and(self.knights(to_move));
        if !knight_attacks.empty() {
            attacks = attacks | knight_attacks;
        }

        // For pawns, there are only a few places a pawn could be to legally attack this square. In all cases,
        // the capturing pawn has to be on the rank immediately above (or below) the square we're looking at.
        //
        // A correlary to this is that pieces on the bottom (or top) ranks can't be attacked by pawns.
        let cant_be_attacked_by_pawns_rank = if to_move == Color::White {
            Rank::One
        } else {
            Rank::Eight
        };

        if target.rank() != cant_be_attacked_by_pawns_rank {
            let pawn_attack_rank = if to_move == Color::White {
                target.towards(Direction::South).rank()
            } else {
                target.towards(Direction::North).rank()
            };

            for pawn in self.pawns(to_move) & Bitboard::all().rank(pawn_attack_rank) {
                if attacks::pawn_attacks(pawn, to_move).test(target) {
                    attacks.set(pawn);
                }
            }
        }

        // There's only one king, so it's cheap to check.
        for king in self.kings(to_move) {
            if attacks::king_attacks(king).test(target) {
                attacks.set(king);
            }
        }

        attacks
    }

    pub fn is_check(&self, color: Color) -> bool {
        for king in self.kings(color) {
            if !self.squares_attacking(color.toggle(), king).empty() {
                return true;
            }
        }

        false
    }

    /// Returns whether or not the piece at the given square is absolutely pinned. If there is no
    /// piece at the given square, or the piece does not belong to the moving player, this method
    /// returns false.
    pub fn is_absolutely_pinned(&self, to_move: Color, sq: Square) -> bool {
        // If there's no piece at this square, there is no pin.
        let pinned_piece = match self.piece_at(sq) {
            Some(piece) => piece,
            None => return false,
        };

        if pinned_piece.kind == PieceKind::King {
            // Kings can't be pinned.
            return false;
        }

        // Who's attacking this square? If nobody, we're not pinned.
        let attacks = self.squares_attacking(to_move, sq);
        if attacks.empty() {
            return false;
        }

        // If this piece is pinned, one of the pieces attacking this square will also attack the
        // king if the piece is removed. The basic strategy here is to simulate an attack if the
        // maybe-pinned piece is removed; if the king is attacked, the given piece must be pinned.
        let kings = self.kings(to_move.toggle());
        let mut all_pieces_except_pin = self.pieces(Color::White).or(self.pieces(Color::Black));
        let all_pieces = all_pieces_except_pin;
        all_pieces_except_pin.unset(sq);
        for attack in attacks {
            // If this piece slides and can now attack a king, the piece on the given square must
            // be absolutely pinned.
            let piece = self.piece_at(attack).unwrap();
            if piece.is_sliding()
                && !piece
                    .attacks(attack, all_pieces_except_pin)
                    .and(kings)
                    .empty()
            {
                // Could we attack this piece before removing? If not, this is a pin.
                if piece.attacks(attack, all_pieces).and(kings).empty() {
                    return true;
                }
            }
        }

        false
    }

    /// Move legality test. Returns true if this move is a legal move from the given position. If
    /// the move is know to be psuedolegal, `is_legal_given_pseudolegal` will likely be faster.
    pub fn is_legal(&self, mov: Move) -> bool {
        let mut mov_vec = MoveVec::default();
        let gen = MoveGenerator::new();
        gen.generate_moves(self, &mut mov_vec);
        if !mov_vec.contains(&mov) {
            return false;
        }

        self.is_legal_given_pseudolegal(mov)
    }

    /// Legality test for moves that are already known to be pseudolegal. This is strictly faster
    /// than `is_legal`, since `is_legal` also needs to check for pseudo-legality. This method is
    /// useful for legality testing moves coming out of the move generator, which is known to
    /// produce only pseudolegal moves.
    pub fn is_legal_given_pseudolegal(&self, mov: Move) -> bool {
        // The below implementation is naive and simple, but correct. It's also probably faster
        // than the more complicated and incorrect commented-out implementation below.
        let mut new_pos = self.clone();
        let side = self.side_to_move();
        new_pos.apply_move(mov);
        !new_pos.is_check(side)

        /*
        let moving_piece = match self.piece_at(mov.source()) {
            Some(piece) => piece,
            None => return false, // not legal: no piece at source square
        };

        // At this point we've confirmed that the move is pseudolegal. For a move to
        // be legal, it must not leave the king in check after the move. This has two
        // meanings, depending on the current state of the board:
        //
        //   1. If the player to move is in check, the given move must leave them out
        //   of check
        //   2. If the player to move is not in check, the given move must keep them
        //   out of check
        //
        // Therefore this function proceeds differently depending on whether or not
        // we're in check.
        let to_move = self.side_to_move();
        if self.is_check(to_move) {
            for king in self.kings(to_move) {
                let checking_pieces = self.squares_attacking(to_move.toggle(), king);
                if checking_pieces.count() > 1 {
                    // Double (or more) check. Only the king is allowed to move in double check.
                    if moving_piece.kind != PieceKind::King {
                        return false;
                    }

                // Fall-through to the remainder of this function. We're moving a king;
                // the rest of the function will validate that the king's not moving to a
                // checked square.
                } else {
                    assert!(
                        checking_pieces.count() == 1,
                        "should be exactly one checking piece"
                    );
                    let checking_piece_square = checking_pieces.first().unwrap();
                    let checking_piece = self
                        .piece_at(checking_piece_square)
                        .expect("no checking piece despite being in squares attacking");

                    // We're being checked by exactly one piece. There are three options
                    // available to us:
                    //   1. Capture the checking piece.
                    //   2. Block the checking piece.
                    //   3. Move the king out of danger.
                    if moving_piece.kind != PieceKind::King {
                        // If we're moving to a piece other than where the checking piece
                        // resides, our intention is to block the checking piece.
                        if mov.destination() != checking_piece_square && !mov.is_en_passant() {
                            // Like the pin detection routine, the idea here is to create a bitboard
                            // representing the board if we made the move, so we can check and see
                            // if we're still in check afterwards.
                            let mut all_pieces_with_block =
                                self.pieces(Color::White).or(self.pieces(Color::Black));
                            all_pieces_with_block.unset(mov.source());
                            all_pieces_with_block.set(mov.destination());
                            if checking_piece
                                .attacks(checking_piece_square, all_pieces_with_block)
                                .test(king)
                            {
                                // This move isn't legal if we're still in check afterwards.
                                return false;
                            }
                        } else if mov.is_en_passant() {
                        }
                    }
                }
            }
        }

        // At this point we have validated that the candidate move gets us out of
        // check, if we are in check. Next we must validate that the candidate move
        // doesn't put is back in check.
        //
        // This can happen in three ways:
        //   1. The piece being moved is a king and the king moved into check
        //   2. A piece that was absolutely pinned moved
        //   3. An en-passant move captured a piece, removing two pieces from the same
        //   rank and leaving an attack on the king.
        //
        // If we're not in check, we're fine as long as we don't move into check.
        // If this is a king move, does it move into check?
        if moving_piece.kind == PieceKind::King {
            let mut cloned = self.clone();
            cloned.apply_move(mov);
            if cloned.is_check(to_move) {
                return false;
            }
        }

        // Is this piece absolutely pinned to the king?
        if self.is_absolutely_pinned(to_move.toggle(), mov.source()) {
            // If it is, does making the move leave the king checked?
            //
            // This is super simple , but hopefully we don't get pinned often.
            let mut cloned = self.clone();
            cloned.apply_move(mov);
            if cloned.is_check(to_move) {
                return false;
            }
        }

        // Otherwise, all good!
        true
        */
    }
}

//
// FEN and UCI parsing and generation.
//
// The routines in this block are oriented around FEN, a simple notation for chess positions.
// Positions can be created by parsing FEN and FEN can be produced from particular positions.
//
// UCI move parsing is also done here. It is not necessarily straightforward to derive a Move
// representation from a UCI move string; it requires full knowledge of the current position to
// disambiguate a move.
//

impl Position {
    pub fn from_start_position() -> Position {
        Position::from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1").unwrap()
    }

    /// Constructs a new position from a FEN representation of a board position.
    pub fn from_fen<S: AsRef<str>>(fen: S) -> Result<Position, FenParseError> {
        use std::iter::Peekable;
        use std::str::Chars;

        type Stream<'a> = Peekable<Chars<'a>>;

        fn eat<'a>(iter: &mut Stream<'a>, expected: char) -> Result<(), FenParseError> {
            match iter.next() {
                Some(c) if c == expected => Ok(()),
                Some(c) => Err(FenParseError::UnexpectedChar(c)),
                None => Err(FenParseError::UnexpectedEnd),
            }
        }

        fn advance<'a>(iter: &mut Stream<'a>) -> Result<(), FenParseError> {
            let _ = iter.next();
            Ok(())
        }

        fn peek<'a>(iter: &mut Stream<'a>) -> Result<char, FenParseError> {
            if let Some(c) = iter.peek() {
                Ok(*c)
            } else {
                Err(FenParseError::UnexpectedEnd)
            }
        }

        fn eat_side_to_move<'a>(iter: &mut Stream<'a>) -> Result<Color, FenParseError> {
            let side = match peek(iter)? {
                'w' => Color::White,
                'b' => Color::Black,
                _ => return Err(FenParseError::InvalidSideToMove),
            };

            advance(iter)?;
            Ok(side)
        }

        fn eat_castle_status<'a>(iter: &mut Stream<'a>) -> Result<CastleStatus, FenParseError> {
            if peek(iter)? == '-' {
                advance(iter)?;
                return Ok(CastleStatus::NONE);
            }

            let mut status = CastleStatus::NONE;
            for _ in 0..4 {
                match peek(iter)? {
                    'K' => status |= CastleStatus::WHITE_KINGSIDE,
                    'k' => status |= CastleStatus::BLACK_KINGSIDE,
                    'Q' => status |= CastleStatus::WHITE_QUEENSIDE,
                    'q' => status |= CastleStatus::BLACK_QUEENSIDE,
                    ' ' => break,
                    _ => return Err(FenParseError::InvalidCastle),
                }

                advance(iter)?;
            }

            Ok(status)
        }

        fn eat_en_passant<'a>(iter: &mut Stream<'a>) -> Result<Option<Square>, FenParseError> {
            let c = peek(iter)?;
            if c == '-' {
                advance(iter)?;
                return Ok(None);
            }

            if let Ok(file) = File::try_from(c) {
                advance(iter)?;
                let rank_c = peek(iter)?;
                if let Ok(rank) = Rank::try_from(rank_c) {
                    advance(iter)?;
                    Ok(Some(Square::of(rank, file)))
                } else {
                    Err(FenParseError::InvalidEnPassant)
                }
            } else {
                Err(FenParseError::InvalidEnPassant)
            }
        }

        fn eat_halfmove<'a>(iter: &mut Stream<'a>) -> Result<u32, FenParseError> {
            let mut buf = String::new();
            loop {
                let c = peek(iter)?;
                if !c.is_digit(10) {
                    break;
                }

                buf.push(c);
                advance(iter)?;
            }

            if buf.is_empty() {
                return Err(FenParseError::EmptyHalfmove);
            }

            buf.parse::<u32>()
                .map_err(|_| FenParseError::InvalidHalfmove)
        }

        fn eat_fullmove<'a>(iter: &mut Stream<'a>) -> Result<u32, FenParseError> {
            let mut buf = String::new();
            for ch in iter {
                if !ch.is_digit(10) {
                    if buf.is_empty() {
                        return Err(FenParseError::EmptyFullmove);
                    }

                    break;
                }

                buf.push(ch);
            }

            if buf.is_empty() {
                return Err(FenParseError::EmptyFullmove);
            }

            buf.parse::<u32>()
                .map_err(|_| FenParseError::InvalidFullmove)
        }

        let mut pos = Position::new();
        let str_ref = fen.as_ref();
        let iter = &mut str_ref.chars().peekable();
        for &rank in RANKS.iter().rev() {
            let mut file = File::A as usize;
            while file <= File::H as usize {
                let c = peek(iter)?;
                // digits 1 through 8 indicate empty squares.
                if c.is_digit(10) {
                    if c < '1' || c > '8' {
                        return Err(FenParseError::InvalidDigit);
                    }

                    let value = c as usize - 48;
                    file += value;
                    if file > 8 {
                        return Err(FenParseError::FileDoesNotSumToEight);
                    }

                    advance(iter)?;
                    continue;
                }

                // if it's not a digit, it represents a piece.
                let piece = if let Ok(piece) = Piece::try_from(c) {
                    piece
                } else {
                    return Err(FenParseError::UnknownPiece);
                };

                let square = Square::of(rank, File::from_index(file));
                pos.add_piece(square, piece).expect("FEN double-add piece?");
                advance(iter)?;
                file += 1;
            }

            if rank != Rank::One {
                eat(iter, '/')?;
            }
        }

        eat(iter, ' ')?;
        pos.side_to_move = eat_side_to_move(iter)?;
        eat(iter, ' ')?;
        pos.castle_status = eat_castle_status(iter)?;
        eat(iter, ' ')?;
        pos.en_passant_square = eat_en_passant(iter)?;
        eat(iter, ' ')?;
        pos.halfmove_clock = eat_halfmove(iter)?;
        eat(iter, ' ')?;
        pos.fullmove_clock = eat_fullmove(iter)?;
        pos.zobrist_hash = zobrist::hash(&pos);
        Ok(pos)
    }

    /// Parses the UCI representation of a move into a Move object, suitable as an argument to
    /// `apply_move`.
    pub fn move_from_uci(&self, move_str: &str) -> Option<Move> {
        // UCI encodes a move as the source square, followed by the destination
        // square, and optionally followed by the promotion piece if necessary.
        let move_chrs: Vec<_> = move_str.chars().collect();
        if move_chrs.len() < 4 {
            // It's not a valid move encoding at all if it's this short.
            return None;
        }

        // A particular quirk of UCI is that null moves are encoded as 0000.
        if move_str == "0000" {
            return Some(Move::null());
        }

        let source_file = File::try_from(move_chrs[0]).ok()?;
        let source_rank = Rank::try_from(move_chrs[1]).ok()?;
        let dest_file = File::try_from(move_chrs[2]).ok()?;
        let dest_rank = Rank::try_from(move_chrs[3]).ok()?;
        let maybe_promotion_piece = if move_chrs.len() == 5 {
            Some(move_chrs[4])
        } else {
            None
        };

        let source = Square::of(source_rank, source_file);
        let dest = Square::of(dest_rank, dest_file);

        // This method is annoyingly complex, so read this here first!
        //
        // We're going to assume that a move is quiet if it's not any other category
        // of move. This means that we might not produce a legal move, but it's up
        // to the legality tests later on to make sure that this move is legit.
        //
        // There are a bunch of cases here that we have to handle. They are encoded
        // in this decision tree:
        // 1. Is the moving piece a pawn?
        //   1.1. Is the moving square two squares straight ahead? => DoublePawnPush
        //   1.2. Is the moving square a legal attack for a pawn?
        //     1.2.1. Is the destination square on a promotion rank? =>
        //     PromotionCapture
        //     1.2.2. Is the destination square the en-passant square?
        //     => EnPassant
        //     1.2.3. else => Capture
        //   1.3. Is the destination square on a promotion rank? =? Promotion
        //   1.4. else => Quiet
        // 2. Is the moving piece a king?
        //   2.1. Is the target the square to the right of the kingside rook? =>
        //   KingsideCastle
        //   2.2. Is the target the square to the right of the queenside rook? =>
        //   QueensideCastle
        //   2.3. Is there piece on the target square? => Capture
        //   2.4. else => Quiet
        // 3. Is there a piece on the target square? => Capture
        // 4. else => Quiet
        //
        // Whew!
        let dest_piece = self.piece_at(dest);
        let moving_piece = self.piece_at(source)?;

        // 1. Is the moving piece a pawn?
        if moving_piece.kind == PieceKind::Pawn {
            let (pawn_dir, promo_rank, start_rank) = match self.side_to_move {
                Color::White => (Direction::North, BB_RANK_8, BB_RANK_2),
                Color::Black => (Direction::South, BB_RANK_1, BB_RANK_7),
            };

            // 1.1. Is the moving square two squares straight ahead?
            if start_rank.test(source) {
                let double_pawn_square = source.towards(pawn_dir).towards(pawn_dir);
                if double_pawn_square == dest {
                    return Some(Move::double_pawn_push(source, dest));
                }
            }

            // 1.2. Is the moving square a legal attack for a pawn?
            if attacks::pawn_attacks(source, self.side_to_move).test(dest) {
                // 1.2.1. Is the destination square on a promotion rank?
                if promo_rank.test(dest) {
                    let promo_piece = maybe_promotion_piece?;
                    let kind = match promo_piece {
                        'n' => PieceKind::Knight,
                        'b' => PieceKind::Bishop,
                        'r' => PieceKind::Rook,
                        'q' => PieceKind::Queen,
                        _ => return None,
                    };

                    return Some(Move::promotion_capture(source, dest, kind));
                }

                // 1.2.2. Is the destination square the en-passant square?
                if Some(dest) == self.en_passant_square {
                    return Some(Move::en_passant(source, dest));
                }

                // 1.2.3. Else, it's a capture.
                return Some(Move::capture(source, dest));
            }

            // 1.3. Is the destination square on a promotion rank?
            if promo_rank.test(dest) {
                let promo_piece = maybe_promotion_piece?;
                let kind = match promo_piece {
                    'n' => PieceKind::Knight,
                    'b' => PieceKind::Bishop,
                    'r' => PieceKind::Rook,
                    'q' => PieceKind::Queen,
                    _ => return None,
                };

                return Some(Move::promotion(source, dest, kind));
            }

            // 1.4. Else, it's a quiet move.
            return Some(Move::quiet(source, dest));
        }

        // 2. Is the moving piece a king?
        if moving_piece.kind == PieceKind::King {
            let (kingside_rook_adjacent, queenside_rook_adjacent, king_start) =
                match self.side_to_move {
                    Color::White => (Square::G1, Square::C1, Square::E1),
                    Color::Black => (Square::G8, Square::C8, Square::E8),
                };

            if king_start == source {
                // 2.1. Is the target of the square to the left of the kingside rook?
                if kingside_rook_adjacent == dest {
                    return Some(Move::kingside_castle(source, dest));
                }

                // 2.2. Is the target the square to the right of the queenside rook?
                if queenside_rook_adjacent == dest {
                    return Some(Move::queenside_castle(source, dest));
                }
            }

            // 2.3. Is there a piece on the target square?
            if dest_piece.is_some() {
                return Some(Move::capture(source, dest));
            }

            // 2.4. Else, it's quiet.
            return Some(Move::quiet(source, dest));
        }

        // 3. Is there a piece on the target square?
        if dest_piece.is_some() {
            return Some(Move::capture(source, dest));
        }

        // 4. Else, it's quiet.
        return Some(Move::quiet(source, dest));
    }

    pub fn as_fen(&self) -> String {
        let mut buf = String::new();
        for &rank in RANKS.iter().rev() {
            let mut empty_squares = 0;
            for &file in &FILES {
                let square = Square::of(rank, file);
                if let Some(piece) = self.piece_at(square) {
                    if empty_squares != 0 {
                        write!(&mut buf, "{}", empty_squares).unwrap();
                    }
                    write!(&mut buf, "{}", piece).unwrap();
                    empty_squares = 0;
                } else {
                    empty_squares += 1;
                }
            }

            if empty_squares != 0 {
                write!(&mut buf, "{}", empty_squares).unwrap();
            }

            if rank != Rank::One {
                buf.push('/');
            }
        }

        buf.push(' ');
        match self.side_to_move() {
            Color::White => buf.push('w'),
            Color::Black => buf.push('b'),
        }
        buf.push(' ');
        if self.can_castle_kingside(Color::White) {
            buf.push('K');
        }
        if self.can_castle_queenside(Color::White) {
            buf.push('Q');
        }
        if self.can_castle_kingside(Color::Black) {
            buf.push('k');
        }
        if self.can_castle_queenside(Color::Black) {
            buf.push('q');
        }
        buf.push(' ');
        if let Some(ep_square) = self.en_passant_square() {
            write!(&mut buf, "{}", ep_square).unwrap();
        } else {
            buf.push('-');
        }
        buf.push(' ');
        write!(
            &mut buf,
            "{} {}",
            self.halfmove_clock(),
            self.fullmove_clock()
        )
        .unwrap();
        buf
    }
}

//
// Trait implementations
//

impl fmt::Display for Position {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for &rank in RANKS.iter().rev() {
            for &file in &FILES {
                let sq = Square::of(rank, file);
                if let Some(piece) = self.piece_at(sq) {
                    write!(f, " {} ", piece)?;
                } else {
                    write!(f, " . ")?;
                }
            }

            writeln!(f, "| {}", rank)?;
        }

        for _ in &FILES {
            write!(f, "---")?;
        }

        writeln!(f)?;
        for &file in &FILES {
            write!(f, " {} ", file)?;
        }

        writeln!(f)?;
        Ok(())
    }
}

impl Default for Position {
    fn default() -> Self {
        Position::new()
    }
}

//
// Helper functions
//

fn kingside_rook(color: Color) -> Square {
    match color {
        Color::White => Square::H1,
        Color::Black => Square::H8,
    }
}

fn kingside_castle_mask(color: Color) -> CastleStatus {
    match color {
        Color::White => CastleStatus::WHITE_KINGSIDE,
        Color::Black => CastleStatus::BLACK_KINGSIDE,
    }
}

fn queenside_rook(color: Color) -> Square {
    match color {
        Color::White => Square::A1,
        Color::Black => Square::A8,
    }
}

fn queenside_castle_mask(color: Color) -> CastleStatus {
    match color {
        Color::White => CastleStatus::WHITE_QUEENSIDE,
        Color::Black => CastleStatus::BLACK_QUEENSIDE,
    }
}

fn castle_mask(color: Color) -> CastleStatus {
    match color {
        Color::White => CastleStatus::WHITE,
        Color::Black => CastleStatus::BLACK,
    }
}

#[cfg(test)]
mod tests {
    use std::mem;

    use crate::moves::Move;
    use crate::position::Position;
    use crate::types::{Color, Square};

    #[test]
    fn size_is_136() {
        assert_eq!(136, mem::size_of::<Position>());
    }

    #[test]
    fn check_smoke() {
        let pos =
            Position::from_fen("r2q1rk1/pP1p2pp/Q4n2/bbp1p3/Np6/1B3NBn/pPPP1PPP/R3K2R b KQ - 0 1")
                .unwrap();
        assert!(pos.is_check(Color::Black));
    }

    #[test]
    fn absolute_pin_smoke() {
        let pos = Position::from_fen("8/8/3q4/8/8/3B4/3K4/8 w - - 0 1").unwrap();
        assert!(pos.is_absolutely_pinned(Color::Black, Square::D3));
    }

    #[test]
    fn absolute_pin_smoke_neg() {
        let pos = Position::from_fen("8/8/3q4/8/8/3B4/2K5/8 w - - 0 1").unwrap();
        assert!(!pos.is_absolutely_pinned(Color::Black, Square::D3));
    }

    #[test]
    fn absolute_pin_smoke_neg_2() {
        let pos = Position::from_fen("8/8/3q4/8/1K6/3B4/8/8 w - - 0 1").unwrap();
        assert!(!pos.is_absolutely_pinned(Color::Black, Square::D3));
    }

    #[test]
    fn absolute_pin_legality() {
        let pos = Position::from_fen("8/8/8/q7/8/2B5/3K4/8 w - - 0 1").unwrap();
        assert!(pos.is_legal(Move::quiet(Square::C3, Square::B4)));
    }

    mod fen {
        use std::convert::TryFrom;

        use crate::moves::Move;
        use crate::types::TableIndex;
        use crate::types::{Color, File, Piece, PieceKind, Rank, Square};

        use crate::position::{FenParseError, Position};

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
            let pos =
                Position::from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1")
                    .unwrap();

            let check_square = |square: &'static str, piece: Piece| {
                assert!(square.len() == 2);
                let chars: Vec<_> = square.chars().collect();
                let file = File::try_from(chars[0]).unwrap();
                let rank = Rank::try_from(chars[1]).unwrap();
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
                let sq_actual = Square::from_index(sq);
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

        #[test]
        fn uci_nullmove() {
            let pos = Position::from_start_position();
            assert_eq!(Move::null(), pos.move_from_uci("0000").unwrap());
        }

        #[test]
        fn uci_sliding_moves() {
            let pos = Position::from_fen("8/3q4/8/8/8/3R4/8/8 w - - 0 1").unwrap();
            assert_eq!(
                Move::quiet(Square::D3, Square::D5),
                pos.move_from_uci("d3d5").unwrap()
            );
            assert_eq!(
                Move::capture(Square::D3, Square::D7),
                pos.move_from_uci("d3d7").unwrap()
            );
        }

        #[test]
        fn uci_pawn_moves() {
            let pos = Position::from_fen("8/8/8/8/8/4p3/3P4/8 w - c3 0 1").unwrap();
            assert_eq!(
                Move::quiet(Square::D2, Square::D3),
                pos.move_from_uci("d2d3").unwrap()
            );
            assert_eq!(
                Move::double_pawn_push(Square::D2, Square::D4),
                pos.move_from_uci("d2d4").unwrap()
            );
            assert_eq!(
                Move::capture(Square::D2, Square::E3),
                pos.move_from_uci("d2e3").unwrap()
            );
            assert_eq!(
                Move::quiet(Square::D2, Square::D3),
                pos.move_from_uci("d2d3").unwrap()
            );
            assert_eq!(
                Move::en_passant(Square::D2, Square::C3),
                pos.move_from_uci("d2c3").unwrap()
            );
        }

        #[test]
        fn uci_king_moves() {
            let pos = Position::from_fen("8/8/8/8/8/8/3r4/R3K2R w - - 0 1").unwrap();
            assert_eq!(
                Move::kingside_castle(Square::E1, Square::G1),
                pos.move_from_uci("e1g1").unwrap(),
            );
            assert_eq!(
                Move::queenside_castle(Square::E1, Square::C1),
                pos.move_from_uci("e1c1").unwrap(),
            );
            assert_eq!(
                Move::quiet(Square::E1, Square::E2),
                pos.move_from_uci("e1e2").unwrap(),
            );
            assert_eq!(
                Move::capture(Square::E1, Square::D2),
                pos.move_from_uci("e1d2").unwrap(),
            );
        }

        #[test]
        fn uci_promotion() {
            let pos = Position::from_fen("5n2/4P3/8/8/8/8/8/8 w - - 0 1").unwrap();
            assert_eq!(
                Move::promotion(Square::E7, Square::E8, PieceKind::Knight),
                pos.move_from_uci("e7e8n").unwrap()
            );
            assert_eq!(
                Move::promotion(Square::E7, Square::E8, PieceKind::Bishop),
                pos.move_from_uci("e7e8b").unwrap()
            );
            assert_eq!(
                Move::promotion(Square::E7, Square::E8, PieceKind::Rook),
                pos.move_from_uci("e7e8r").unwrap()
            );
            assert_eq!(
                Move::promotion(Square::E7, Square::E8, PieceKind::Queen),
                pos.move_from_uci("e7e8q").unwrap()
            );
            assert_eq!(
                Move::promotion_capture(Square::E7, Square::F8, PieceKind::Knight),
                pos.move_from_uci("e7f8n").unwrap()
            );
            assert_eq!(
                Move::promotion_capture(Square::E7, Square::F8, PieceKind::Bishop),
                pos.move_from_uci("e7f8b").unwrap()
            );
            assert_eq!(
                Move::promotion_capture(Square::E7, Square::F8, PieceKind::Rook),
                pos.move_from_uci("e7f8r").unwrap()
            );
            assert_eq!(
                Move::promotion_capture(Square::E7, Square::F8, PieceKind::Queen),
                pos.move_from_uci("e7f8q").unwrap()
            );
        }
    }

    mod apply {
        use crate::position::Position;

        use crate::moves::Move;
        use crate::types::{Color, PieceKind, Square};

        #[test]
        fn smoke_test_opening_pawn() {
            let mut pos =
                Position::from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 2 1")
                    .unwrap();

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
            pos.apply_move(Move::promotion_capture(
                Square::E7,
                Square::F8,
                PieceKind::Queen,
            ));

            // there should be a white queen on f8
            let queen = pos.piece_at(Square::F8).unwrap();
            assert_eq!(Color::White, queen.color);
            assert_eq!(PieceKind::Queen, queen.kind);
        }

        #[test]
        fn queenside_castle() {
            let mut pos = Position::from_fen("8/8/8/8/8/8/8/R3K3 w Q - 0 1").unwrap();

            // white to move, white castles queenside
            pos.apply_move(Move::queenside_castle(Square::E1, Square::C1));

            let rook = pos.piece_at(Square::D1).unwrap();
            assert_eq!(Color::White, rook.color);
            assert_eq!(PieceKind::Rook, rook.kind);

            let king = pos.piece_at(Square::C1).unwrap();
            assert_eq!(Color::White, king.color);
            assert_eq!(PieceKind::King, king.kind);
        }

        #[test]
        fn kingside_castle() {
            let mut pos = Position::from_fen("8/8/8/8/8/8/8/4K2R w K - 0 1").unwrap();

            // white to move, white castles kingside
            pos.apply_move(Move::kingside_castle(Square::E1, Square::G1));

            let rook = pos.piece_at(Square::F1).unwrap();
            assert_eq!(Color::White, rook.color);
            assert_eq!(PieceKind::Rook, rook.kind);

            let king = pos.piece_at(Square::G1).unwrap();
            assert_eq!(Color::White, king.color);
            assert_eq!(PieceKind::King, king.kind);
        }
    }
}

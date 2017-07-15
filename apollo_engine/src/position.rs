// Copyright 2017 Sean Gillespie.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.
use std::fmt::{self, Debug, Display, Write};
use std::vec::IntoIter;
use num_traits::FromPrimitive;
use bitboard::Bitboard;
use types::{self, Square, Piece, Color, CastleStatus, Rank, File, PieceKind, Direction};
use moves::Move;
use attacks;
use movegen;

/// Possible errors that can arise when parsing a FEN string into
/// a `Position`.
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

/// A Position encapulsates all of the state of a single position in chess. It
/// contains all informatio necessary to compute legal moves and advance the
/// game. Moves can be applied to Positions to advance the game state.
#[derive(Clone)]
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
    pub const fn new() -> Position {
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
    ///
    /// # Example
    /// ```
    /// use apollo_engine::{Position, Square};
    ///
    /// let pos = Position::from_fen("8/8/8/8/8/8/2R5/8 w KQkq - 0 1").unwrap();
    /// assert!(pos.piece_at(Square::C2).is_some());
    /// ```
    pub fn from_fen<S: AsRef<str>>(fen: S) -> Result<Position, FenParseError> {
        use std::str::Chars;
        use std::iter::Peekable;

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
                return Ok(types::CASTLE_NONE);
            }

            let mut status = types::CASTLE_NONE;
            for _ in 0..4 {
                match peek(iter)? {
                    'K' => status |= types::WHITE_O_O,
                    'k' => status |= types::BLACK_O_O,
                    'Q' => status |= types::WHITE_O_O_O,
                    'q' => status |= types::BLACK_O_O_O,
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

            if let Some(file) = File::from_char(c) {
                advance(iter)?;
                let rank_c = peek(iter)?;
                if let Some(rank) = Rank::from_char(rank_c) {
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

            if buf.len() == 0 {
                return Err(FenParseError::EmptyHalfmove);
            }

            buf.parse::<u32>().map_err(|_| FenParseError::InvalidHalfmove)
        }

        fn eat_fullmove<'a>(iter: &mut Stream<'a>) -> Result<u32, FenParseError> {
            let mut buf = String::new();
            while let Some(ch) = iter.next() {
                if !ch.is_digit(10) {
                    if buf.len() == 0 {
                        return Err(FenParseError::EmptyFullmove);
                    }

                    break;
                }

                buf.push(ch);
            }

            if buf.len() == 0 {
                return Err(FenParseError::EmptyFullmove);
            }

            buf.parse::<u32>().map_err(|_| FenParseError::InvalidFullmove)
        }

        let mut pos = Position::new();
        let str_ref = fen.as_ref();
        let ref mut iter = str_ref.chars().peekable();
        for rank in ((Rank::Rank1 as usize)...(Rank::Rank8 as usize)).rev() {
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
                let piece = if let Some(piece) = Piece::from_char(c) {
                    piece
                } else {
                    return Err(FenParseError::UnknownPiece);
                };

                let file_enum = FromPrimitive::from_u64(file as u64).unwrap();
                let rank_enum = FromPrimitive::from_u64(rank as u64).unwrap();
                let square = Square::of(rank_enum, file_enum);
                pos.add_piece(square, piece).expect("FEN double-add piece?");
                advance(iter)?;
                file += 1;
            }

            if rank != (Rank::Rank1 as usize) {
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
        Ok(pos)
    }

    /// Returns a FEN representation of this position.
    ///
    /// # Example
    /// ```
    /// use apollo_engine::Position;
    ///
    /// let fen = "8/8/8/4B3/8/8/8/8 w KQkq - 0 1";
    /// let position = Position::from_fen(fen).unwrap();
    /// assert_eq!(fen, position.as_fen());
    /// ```
    pub fn as_fen(&self) -> String {
        let mut string = String::new();
        for rank_idx in ((Rank::Rank1 as usize)...(Rank::Rank8 as usize)).rev() {
            let rank = FromPrimitive::from_u64(rank_idx as u64).unwrap();
            let mut empty_squares = 0;
            for file_idx in (File::A as usize)...(File::H as usize) {
                let file = FromPrimitive::from_u64(file_idx as u64).unwrap();
                let sq = Square::of(rank, file);
                if let Some(piece) = self.piece_at(sq) {
                    if empty_squares != 0 {
                        write!(&mut string, "{}", empty_squares).unwrap();
                    }

                    string.push(piece.as_char());
                    empty_squares = 0;
                } else {
                    empty_squares += 1;
                }
            }

            if empty_squares != 0 {
                write!(&mut string, "{}", empty_squares).unwrap();
            }
            if rank != Rank::Rank1 {
                string.push('/');
            }
        }

        string.push(' ');
        let side_char = match self.side_to_move {
            Color::White => 'w',
            Color::Black => 'b',
        };
        string.push(side_char);
        string.push(' ');
        let mut someone_can_castle = false;
        if self.can_castle_kingside(Color::White) {
            string.push('K');
            someone_can_castle = true;
        }

        if self.can_castle_queenside(Color::White) {
            string.push('Q');
            someone_can_castle = true;
        }

        if self.can_castle_kingside(Color::Black) {
            string.push('k');
            someone_can_castle = true;
        }

        if self.can_castle_queenside(Color::Black) {
            string.push('q');
            someone_can_castle = true;
        }

        if !someone_can_castle {
            string.push('-');
        }

        string.push(' ');
        if let Some(ep_square) = self.en_passant_square {
            write!(&mut string, "{}", ep_square).unwrap();
        } else {
            string.push('-');
        }

        string.push(' ');
        write!(&mut string,
               "{} {}",
               self.halfmove_clock,
               self.fullmove_clock)
                .unwrap();
        string
    }

    /// Adds a piece to the board at the given square, returning `Ok` if
    /// the adding was successful (i.e. the space was unoccupied) and `Err`
    /// if the space was occupied.
    ///
    /// # Example
    /// ```
    /// use apollo_engine::{Square, Piece, PieceKind, Position, Color};
    ///
    /// let mut position = Position::new();
    /// let piece = Piece::new(PieceKind::Pawn, Color::White);
    /// position.add_piece(Square::E4, piece);
    /// let on_board = position.piece_at(Square::E4).unwrap();
    /// assert_eq!(on_board.kind, PieceKind::Pawn);
    /// assert_eq!(on_board.color, Color::White);
    /// ```
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

    /// Applies a move to the given position, mutating the given position into
    /// a new position with the change in game state applied to it.
    pub fn apply_move(&mut self, mov: Move) {
        // Subroutine for handling piece capture, since some additional checks
        // are required to ensure correctness when capturing rooks on their
        // starting squares. We also don't want rustc to inline this function
        // since the majority of moves aren't captures.
        fn handle_piece_capture(pos: &mut Position, mov: Move) {
            // en-passant is the only case when the piece being captured
            // does not lie on the same square as the move destination.
            let target_square = if mov.is_en_passant() {
                let dir = match pos.side_to_move {
                    Color::White => Direction::South,
                    Color::Black => Direction::North,
                };

                let ep_square = pos.en_passant_square().expect("ep-move without ep-square");
                FromPrimitive::from_i8(ep_square as i8 + dir.as_vector())
                    .expect("ep-capture square off board")
            } else {
                mov.destination()
            };

            let captured_piece = pos.piece_at(target_square).expect("no piece at capture square");
            pos.remove_piece(target_square).expect("no piece at capture square");

            // if we are capturing a rook that has not moved from its initial
            // state (i.e. the opponent could have used it to legally castle),
            // we have to invalidate the opponent's castling rights.
            let opposing_side = pos.side_to_move.toggle();
            if pos.can_castle_kingside(opposing_side) {
                let starting_square = match opposing_side {
                    Color::White => Square::H1,
                    Color::Black => Square::H8,
                };

                if target_square == starting_square {
                    // if the opponent can castle kingside and we just captured
                    // a piece on the kingside rook starting square, we must
                    // have just captured a rook.
                    assert_eq!(PieceKind::Rook, captured_piece.kind);
                    let flag = match opposing_side {
                        Color::White => types::WHITE_O_O,
                        Color::Black => types::BLACK_O_O,
                    };

                    // eliminate the kingside castle.
                    pos.castle_status &= !flag;
                }
            }

            // same deal for queenside castles.
            if pos.can_castle_queenside(opposing_side) {
                let starting_square = match opposing_side {
                    Color::White => Square::A1,
                    Color::Black => Square::A8,
                };

                if target_square == starting_square {
                    assert_eq!(PieceKind::Rook, captured_piece.kind);
                    let flag = match opposing_side {
                        Color::White => types::WHITE_O_O_O,
                        Color::Black => types::BLACK_O_O_O,
                    };

                    // eliminate the queenside castle.
                    pos.castle_status &= !flag;
                }
            }
        }

        let moving_piece =
            self.piece_at(mov.source()).expect("moving from a square with no piece on it");
        assert_eq!(self.side_to_move,
                   moving_piece.color,
                   "moving a piece that does not belong to the player");

        // the basic strategy here is to remove the piece from the start square
        // and add it to the target square, removing the piece at the target
        // square if this is a capture.
        self.remove_piece(mov.source()).expect("source square has no piece");
        if mov.is_capture() {
            handle_piece_capture(self, mov);
        }

        if mov.is_kingside_castle() || mov.is_queenside_castle() {
            // castles are encoded based on the king's start and stop position.
            // notably, the rook is not at the move destination.
            let (new_rook_sq, rook_sq) = {
                // regardless of how we are castling, the rook appears
                // adjacent to the king.
                let (post_castle_dir, pre_castle_dir, squares) = if mov.is_kingside_castle() {
                    (Direction::West, Direction::East, 1)
                } else {
                    (Direction::East, Direction::West, 2)
                };

                let vec = post_castle_dir.as_vector();
                let new_rook_sq = FromPrimitive::from_i8(mov.destination() as i8 + vec)
                    .expect("rook castle square not on board");

                let other_vec = pre_castle_dir.as_vector() * squares;
                let rook_sq = FromPrimitive::from_i8(mov.destination() as i8 + other_vec)
                    .expect("rook castle source square not on board");
                (new_rook_sq, rook_sq)
            };

            let rook = self.piece_at(rook_sq).expect("rook not at destination");
            assert_eq!(PieceKind::Rook, rook.kind);
            self.remove_piece(rook_sq).expect("empty castle destination");
            // add the rook back at the rook castle location.
            self.add_piece(new_rook_sq, rook).expect("rook castle square occupied");
        }

        let piece_to_add = if mov.is_promotion() {
            Piece::new(mov.promotion_piece(), moving_piece.color)
        } else {
            moving_piece
        };

        self.add_piece(mov.destination(), piece_to_add).expect("destination square was not empty");
        if mov.is_double_pawn_push() {
            // double pawn pushes set the EP-square
            let ep_dir = match self.side_to_move {
                Color::White => Direction::South,
                Color::Black => Direction::North,
            };

            let sq = FromPrimitive::from_i8(mov.destination() as i8 + ep_dir.as_vector())
                .expect("ep-square not on board");
            self.en_passant_square = Some(sq);
        } else {
            self.en_passant_square = None;
        }

        if self.can_castle_kingside(self.side_to_move) ||
           self.can_castle_queenside(self.side_to_move) {
            match moving_piece.kind {
                PieceKind::King => {
                    // if it's the king that's moving, we can't castle in
                    // either direction anymore.
                    let mask = match self.side_to_move {
                        Color::White => types::WHITE_MASK,
                        Color::Black => types::BLACK_MASK,
                    };

                    self.castle_status &= !mask;
                }
                PieceKind::Rook => {
                    let (kingside_rook, queenside_rook) = match self.side_to_move {
                        Color::White => (Square::H1, Square::A1),
                        Color::Black => (Square::H8, Square::A8),
                    };

                    if self.can_castle_queenside(self.side_to_move) &&
                       mov.source() == queenside_rook {
                        let mask = match self.side_to_move {
                            Color::White => types::WHITE_O_O_O,
                            Color::Black => types::BLACK_O_O_O,
                        };

                        self.castle_status &= !mask;
                    }

                    if self.can_castle_kingside(self.side_to_move) &&
                       mov.source() == kingside_rook {
                        let mask = match self.side_to_move {
                            Color::White => types::WHITE_O_O,
                            Color::Black => types::BLACK_O_O,
                        };

                        self.castle_status &= !mask;
                    }
                }
                // other moves don't influence castle status.
                _ => {}
            }
        }

        self.side_to_move = self.side_to_move.toggle();
        if mov.is_capture() || moving_piece.kind == PieceKind::Pawn {
            self.halfmove_clock = 0;
        } else {
            // not capturing or moving a pawn counts against the fifty
            // move rule.
            self.halfmove_clock += 1;
        }

        if self.side_to_move == Color::White {
            // if it's white's turn to move again, a turn has ended.
            self.fullmove_clock += 1;
        }
    }

    fn pieces(&self, color: Color, kind: PieceKind) -> Bitboard {
        let offset = match color {
            Color::White => 0,
            Color::Black => 6,
        };

        self.boards_by_piece[offset + kind as usize]
    }

    pub fn pawns(&self, color: Color) -> Bitboard {
        self.pieces(color, PieceKind::Pawn)
    }

    pub fn knights(&self, color: Color) -> Bitboard {
        self.pieces(color, PieceKind::Knight)
    }

    pub fn bishops(&self, color: Color) -> Bitboard {
        self.pieces(color, PieceKind::Bishop)
    }

    pub fn rooks(&self, color: Color) -> Bitboard {
        self.pieces(color, PieceKind::Rook)
    }

    pub fn queens(&self, color: Color) -> Bitboard {
        self.pieces(color, PieceKind::Queen)
    }

    pub fn kings(&self, color: Color) -> Bitboard {
        self.pieces(color, PieceKind::King)
    }

    pub fn white(&self) -> Bitboard {
        self.color(Color::White)
    }

    pub fn black(&self) -> Bitboard {
        self.color(Color::Black)
    }

    pub fn color(&self, color: Color) -> Bitboard {
        self.boards_by_color[color as usize]
    }

    pub fn squares_attacking(&self, color: Color, square: Square) -> Bitboard {
        let occupancy = self.boards_by_color[Color::White as usize] |
                        self.boards_by_color[Color::Black as usize];
        let mut board = Bitboard::none();
        for queen in self.queens(color) {
            if attacks::queen_attacks(queen, occupancy).test(square) {
                board.set(queen);
            }
        }

        for rook in self.rooks(color) {
            if attacks::rook_attacks(rook, occupancy).test(square) {
                board.set(rook);
            }
        }

        for bishop in self.bishops(color) {
            if attacks::bishop_attacks(bishop, occupancy).test(square) {
                board.set(bishop);
            }
        }

        for knight in self.knights(color) {
            if attacks::knight_attacks(knight).test(square) {
                board.set(knight);
            }
        }

        for pawn in self.pawns(color) {
            if attacks::pawn_attacks(pawn, color).test(square) {
                board.set(pawn);
            }

            if let Some(ep_square) = self.en_passant_square {
                if ep_square == square && attacks::pawn_attacks(pawn, color).test(ep_square) {
                    board.set(pawn);
                }
            }
        }

        for opp_king in self.kings(color) {
            if attacks::king_attacks(opp_king).test(square) {
                board.set(opp_king);
            }
        }

        board
    }

    /// Whether or not the player whose turn it is to move is in check.
    pub fn is_check(&self, color: Color) -> bool {
        let kings = self.kings(color);

        // if there's no king (e.g. unit test or puzzle scenario),
        // there's no check
        if kings.count() == 0 {
            return false;
        }

        // if there are kings on the board, there should be one of them
        assert!(kings.count() == 1);
        let king = kings.first().unwrap();

        // the player whose king this belongs to is in check if there exists
        // an attack by the opponent that includes this square.
        !self.squares_attacking(color.toggle(), king).empty()
    }

    pub fn pseudolegal_moves(&self) -> IntoIter<Move> {
        // it seems to be the case that there does not exist
        // a chess position with more than 218 moves available
        // to it.
        let mut moves = Vec::with_capacity(218);
        movegen::generate_moves(self, &mut moves);
        moves.into_iter()
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

impl Debug for Position {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        let fen = self.as_fen();
        f.debug_tuple("Position").field(&fen).finish()
    }
}

impl Display for Position {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        for rank_idx in ((Rank::Rank1 as u8)...(Rank::Rank8 as u8)).rev() {
            let rank: Rank = FromPrimitive::from_u8(rank_idx).unwrap();
            for file_idx in (File::A as u8)...(File::H as u8) {
                let file: File = FromPrimitive::from_u8(file_idx).unwrap();
                let sq = Square::of(rank, file);
                if let Some(piece) = self.piece_at(sq) {
                    write!(f, " {} ", piece.as_char())?;
                } else {
                    write!(f, " . ")?;
                }
            }

            writeln!(f, "| {}", (rank_idx + 49) as char)?
        }

        for _ in (File::A as u8)...(File::H as u8) {
            write!(f, "---")?;
        }

        writeln!(f, "")?;
        for file_idx in (File::A as u8)...(File::H as u8) {
            write!(f, " {} ", (file_idx + 97) as char)?
        }

        writeln!(f, "")?;
        Ok(())
    }
}
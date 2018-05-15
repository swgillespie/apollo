// Copyright 2017 Sean Gillespie.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.
//! Implementation of Zobrist hashing for Chess positions, used to efficiently
//! and effectively hash positions. Positions are often used as table keys,
//! (e.g. for transposition tables) and as such it's crucial that hashing
//! be as collision-free as possible and also fast.
use num_traits::FromPrimitive;
use position::Position;
use rand::{Rng, SeedableRng, XorShiftRng};
use types::{Color, Piece, PieceKind, Square};

const RANDOM_SEED: [u32; 4] = [0xDEADBEEF, 0xCAFEBABE, 0xBADF00D, 0xFEEEFEEE];
const SIDE_TO_MOVE_INDEX: usize = 768;
const CASTLING_RIGHTS_INDEX: usize = 769;
const EN_PASSANT_INDEX: usize = 773;

pub struct ZobristHasher {
    // [0..767]   - square + piece hashes
    // 768        - side to move hash
    // [769..772] - castling status hashes
    // [773..780] - en passant file hashes
    magic_hashes: [u64; 781],
}

impl ZobristHasher {
    pub fn new() -> ZobristHasher {
        let mut hasher = ZobristHasher {
            magic_hashes: [0; 781],
        };

        hasher.initialize();
        hasher
    }

    fn square_hash(&self, piece: PieceKind, color: Color, square: Square) -> u64 {
        // the layout of the table is:
        //  [square]
        //  0  white pawn hash
        //  1  white knight hash
        //  ...
        //  5  white king hash
        //  6  black pawn hash
        //  7  black knight hash
        //  ...
        //  11 black king hash
        // so, the square base is 12 * square, since the table is laid
        // out as one square after another.
        let square_offset = 12 * square as usize;
        let color_offset = if color == Color::White { 0 } else { 6 };

        let piece_offset = piece as usize;
        return self.magic_hashes[square_offset + color_offset + piece_offset];
    }

    fn side_to_move_hash(&self, color: Color) -> u64 {
        match color {
            Color::White => 0,
            Color::Black => self.magic_hashes[SIDE_TO_MOVE_INDEX],
        }
    }

    fn en_passant_hash(&self, ep_square: Square) -> u64 {
        let file = ep_square.file() as usize;
        return self.magic_hashes[EN_PASSANT_INDEX + file];
    }

    fn castle_hash(&self, offset: usize) -> u64 {
        return self.magic_hashes[CASTLING_RIGHTS_INDEX + offset];
    }

    /// Calculates the Zobrist hash for the given position. This operation
    /// is potentially expensive (it enumerates the entire board) and as such
    /// is only recommended to be called when populating the initial hash value
    /// of starting positions.
    pub fn hash(&self, pos: &Position) -> u64 {
        let mut running_hash = 0;
        for sq_idx in (Square::A1 as usize)..=(Square::H8 as usize) {
            let sq = FromPrimitive::from_u64(sq_idx as u64).unwrap();
            for color in &[Color::White, Color::Black] {
                for piece in &[
                    PieceKind::Pawn,
                    PieceKind::Knight,
                    PieceKind::Bishop,
                    PieceKind::Rook,
                    PieceKind::Queen,
                    PieceKind::King,
                ] {
                    if pos.pieces(*color, *piece).test(sq) {
                        running_hash ^= self.square_hash(*piece, *color, sq);
                    }
                }
            }
        }

        running_hash ^= self.side_to_move_hash(pos.side_to_move());
        if pos.can_castle_kingside(Color::White) {
            running_hash ^= self.castle_hash(0);
        }

        if pos.can_castle_queenside(Color::White) {
            running_hash ^= self.castle_hash(1);
        }

        if pos.can_castle_kingside(Color::Black) {
            running_hash ^= self.castle_hash(2);
        }

        if pos.can_castle_queenside(Color::Black) {
            running_hash ^= self.castle_hash(3);
        }

        if let Some(ep_square) = pos.en_passant_square() {
            running_hash ^= self.en_passant_hash(ep_square);
        }

        running_hash
    }

    /// Incrementally update a Zobrist hash with the information
    /// that a piece has been added or removed from the board.
    pub fn modify_piece(&self, hash: &mut u64, square: Square, piece: Piece) {
        *hash ^= self.square_hash(piece.kind, piece.color, square);
    }

    /// Incrementally update a Zobrist hash with the information
    /// that the side to move has changed.
    pub fn modify_side_to_move(&self, hash: &mut u64) {
        *hash ^= self.side_to_move_hash(Color::Black);
    }

    /// Incrementally update a Zobrist hash with the information
    /// that a player's kingside castle status has changed.
    pub fn modify_kingside_castle(&self, hash: &mut u64, color: Color) {
        let offset = match color {
            Color::White => 0,
            Color::Black => 2,
        };

        *hash ^= self.castle_hash(offset);
    }

    /// Incrementally update a Zobrist hash with the information that
    /// a player's queenside castle status has changed.
    pub fn modify_queenside_castle(&self, hash: &mut u64, color: Color) {
        let offset = match color {
            Color::White => 1,
            Color::Black => 3,
        };

        *hash ^= self.castle_hash(offset);
    }

    /// Incrementally updated a Zobrist hash with the information that
    /// the en-passant square has changed.
    pub fn modify_en_passant(&self, hash: &mut u64, old: Option<Square>, new: Option<Square>) {
        match (old, new) {
            (None, None) => (),
            (Some(sq), None) | (None, Some(sq)) => {
                *hash ^= self.en_passant_hash(sq);
            }
            (Some(sq_old), Some(sq_new)) => {
                *hash ^= self.en_passant_hash(sq_old);
                *hash ^= self.en_passant_hash(sq_new);
            }
        }
    }

    fn initialize(&mut self) {
        let mut rng = XorShiftRng::from_seed(RANDOM_SEED);
        for slot in self.magic_hashes.iter_mut() {
            *slot = rng.next_u64();
        }
    }
}

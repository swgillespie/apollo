// Copyright 2017-2019 Sean Gillespie.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.
use crate::position::Position;
use crate::types::{Color, Piece, PieceKind, Square, TableIndex, COLORS, PIECE_KINDS, SQUARES};

struct Xorshift64 {
    state: u64,
}

impl Xorshift64 {
    pub const fn new(seed: u64) -> Xorshift64 {
        Xorshift64 { state: seed }
    }

    pub fn next(&mut self) -> u64 {
        let mut x = self.state;
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        self.state = x;
        self.state
    }
}

const SIDE_TO_MOVE_INDEX: usize = 768;
const CASTLING_RIGHTS_INDEX: usize = 769;
const EN_PASSANT_INDEX: usize = 773;

struct ZobristHasher {
    rng: Xorshift64,
    magic_hashes: [u64; 781],
}

impl ZobristHasher {
    pub fn new(seed: u64) -> ZobristHasher {
        let mut rng = Xorshift64::new(seed);
        let mut magic_hashes = [0; 781];
        for entry in magic_hashes.iter_mut() {
            *entry = rng.next();
        }

        ZobristHasher { rng, magic_hashes }
    }

    pub fn square_hash(&self, kind: PieceKind, color: Color, square: Square) -> u64 {
        // The layout of this table is:
        // [square]
        //   0 white pawn hash
        //   1 white knight hash
        //   ...
        //   5 white king hash
        //   6 black pawn hash
        //   7 black knight hash
        //   ...
        //   11 black king hash
        //
        // So, the square base is 12 * square, since the table is laid out one
        // square after another.
        let offset = 12 * square.as_index();
        let color_offset = if color == Color::White { 0 } else { 6 };
        let piece_offset = kind.as_index();
        self.magic_hashes[offset + color_offset + piece_offset]
    }

    pub fn side_to_move_hash(&self, side: Color) -> u64 {
        match side {
            Color::White => 0,
            Color::Black => self.magic_hashes[SIDE_TO_MOVE_INDEX],
        }
    }

    pub fn en_passant_hash(&self, square: Square) -> u64 {
        self.magic_hashes[square.file().as_index() + EN_PASSANT_INDEX]
    }

    fn castle_hash(&self, offset: usize) -> u64 {
        self.magic_hashes[offset + CASTLING_RIGHTS_INDEX]
    }

    pub fn hash(&self, pos: &Position) -> u64 {
        let mut running_hash = 0u64;
        for &square in SQUARES.iter() {
            for &color in &COLORS {
                for &kind in &PIECE_KINDS {
                    running_hash ^= self.square_hash(kind, color, square);
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
        if pos.can_castle_kingside(Color::Black) {
            running_hash ^= self.castle_hash(3);
        }
        if let Some(ep_square) = pos.en_passant_square() {
            running_hash ^= self.en_passant_hash(ep_square);
        }
        running_hash
    }
}

const ZOBRIST_SEED: u64 = 0xf68e34a4e8ccf09a;

lazy_static! {
    static ref ZOBRIST_HASHER: ZobristHasher = ZobristHasher::new(ZOBRIST_SEED);
}

pub fn hash(pos: &Position) -> u64 {
    ZOBRIST_HASHER.hash(pos)
}

pub fn modify_piece(hash: &mut u64, square: Square, piece: Piece) {
    *hash ^= ZOBRIST_HASHER.square_hash(piece.kind, piece.color, square);
}

pub fn modify_side_to_move(hash: &mut u64) {
    *hash ^= ZOBRIST_HASHER.side_to_move_hash(Color::Black);
}

pub fn modify_kingside_castle(hash: &mut u64, color: Color) {
    let offset = if color == Color::White { 0 } else { 2 };
    *hash ^= ZOBRIST_HASHER.castle_hash(offset);
}

pub fn modify_queenside_castle(hash: &mut u64, color: Color) {
    let offset = if color == Color::White { 1 } else { 3 };
    *hash ^= ZOBRIST_HASHER.castle_hash(offset);
}

pub fn modify_en_passant(hash: &mut u64, old: Option<Square>, new: Option<Square>) {
    match (old, new) {
        (Some(old), Some(new)) => {
            *hash ^= ZOBRIST_HASHER.en_passant_hash(old);
            *hash ^= ZOBRIST_HASHER.en_passant_hash(new);
        }
        (Some(sq), _) | (_, Some(sq)) => {
            *hash ^= ZOBRIST_HASHER.en_passant_hash(sq);
        }
        _ => {}
    }
}

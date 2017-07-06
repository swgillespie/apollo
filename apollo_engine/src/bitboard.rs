// Copyright 2017 Sean Gillespie. See the COPYRIGHT
// file at the top-level directory of this distribution.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Definitions of the `Bitboard` type, which is conceptually a set of
//! squares on the chess board. Bitboards are used extensively throughout
//! the engine for a number of reasons, for move generation (as in `slides`),
//! check detection, and the locations of all of the pieces on the board.
//!
//! A bitboard is a single 64-bit integer and it behaves like a set, using
//! bitwise operations for the normal set operations (union, intersection,
//! set complement, etc.).
use std::default::Default;
use std::iter::Iterator;
use std::ops;
use std::fmt;
use parking_lot::RwLock;
use types::{Square, Rank, File, Color, Piece, PieceKind};
use slides;
use num_traits::FromPrimitive;

static PAWN_ATTACKS: RwLock<[[Bitboard; 64]; 2]> = RwLock::new([[Bitboard::none(); 64]; 2]);
static KNIGHT_ATTACKS: RwLock<[Bitboard; 64]> = RwLock::new([Bitboard::none(); 64]);
static KING_ATTACKS: RwLock<[Bitboard; 64]> = RwLock::new([Bitboard::none(); 64]);
const RANK_MASKS: [u64; 8] = [0x00000000000000FF,
                              0x000000000000FF00,
                              0x0000000000FF0000,
                              0x00000000FF000000,
                              0x000000FF00000000,
                              0x0000FF0000000000,
                              0x00FF000000000000,
                              0xFF00000000000000];

const FILE_MASKS: [u64; 8] = [0x0101010101010101,
                              0x0202020202020202,
                              0x0404040404040404,
                              0x0808080808080808,
                              0x1010101010101010,
                              0x2020202020202020,
                              0x4040404040404040,
                              0x8080808080808080];

/// A Bitboard is a 64-bit integer which one bit represents one of the
/// eight squares on the board. Bitboards are used in a variety of scenarios
/// to represent the board itself and the pieces upon it.
#[derive(Copy, Clone, Debug)]
pub struct Bitboard {
    bits: u64,
}

impl Default for Bitboard {
    fn default() -> Bitboard {
        Bitboard::none()
    }
}

impl Bitboard {
    /// Constructs a new bitboard from the given bits.
    pub const fn from_bits(bits: u64) -> Bitboard {
        Bitboard { bits: bits }
    }

    /// Constructs a new bitboard with all bits set to one, representing
    /// a complete set.
    pub const fn all() -> Bitboard {
        Bitboard::from_bits(0xFFFFFFFFFFFFFFFF)
    }

    /// Constructs a new bitboard with all bits zeroed, representing
    /// the empty set.
    pub const fn none() -> Bitboard {
        Bitboard::from_bits(0)
    }

    /// Tests whether or not a square is a member of this bitboard.
    pub fn test(&self, square: Square) -> bool {
        (self.bits & (1u64 << (square as u8))) != 0
    }

    /// Sets a square to be a member of this bitboard.
    pub fn set(&mut self, square: Square) {
        self.bits |= 1u64 << (square as u8);
    }

    /// Removes a square from this bitboard.
    pub fn unset(&mut self, square: Square) {
        self.bits &= !(1u64 << square as u8);
    }

    /// Takes the bitwise and of two bitboards producing the set intersection
    /// of their contents.
    pub const fn and(&self, other: Bitboard) -> Bitboard {
        Bitboard::from_bits(self.bits & other.bits)
    }

    /// Takes the bitwise or of two bitboards producing the set union
    /// of their contents.
    pub const fn or(&self, other: Bitboard) -> Bitboard {
        Bitboard::from_bits(self.bits | other.bits)
    }

    /// Takes the bitwise exclusive or of two bitboards.
    pub const fn xor(&self, other: Bitboard) -> Bitboard {
        Bitboard::from_bits(self.bits ^ other.bits)
    }

    /// Takes the bitwise complement of this bitboard, producing the set
    /// complement its contents.
    pub const fn not(&self) -> Bitboard {
        Bitboard::from_bits(!self.bits)
    }

    /// Produces an iterator over the squares contained in this bitboard.
    pub fn iter(&self) -> BitboardIterator {
        BitboardIterator::new(self.bits)
    }

    /// Produces a bitboard with the components of this bitboard that
    /// lie on the given rank.
    pub const fn rank(&self, rank: Rank) -> Bitboard {
        self.and(Bitboard::from_bits(RANK_MASKS[rank as usize]))
    }

    /// Produces a bitboard with the components of this bitboard that
    /// lie on the given file.
    pub const fn file(&self, file: File) -> Bitboard {
        self.and(Bitboard::from_bits(FILE_MASKS[file as usize]))
    }

    /// Retireves the raw bits associated with this bitboard.
    pub const fn bits(&self) -> u64 {
        self.bits
    }

    /// Retrieves the number of squares contained in the set represented
    /// by this bitboard.
    pub fn count(&self) -> u32 {
        self.bits.count_ones()
    }

    /// Retrieves whether or not the set represented by this bitboard is
    /// the empty set.
    pub const fn empty(&self) -> bool {
        self.bits == 0
    }
}

impl fmt::Display for Bitboard {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        for rank_idx in ((Rank::Rank1 as u8)...(Rank::Rank8 as u8)).rev() {
            let rank: Rank = FromPrimitive::from_u8(rank_idx).unwrap();
            for file_idx in (File::A as u8)...(File::H as u8) {
                let file: File = FromPrimitive::from_u8(file_idx).unwrap();
                let sq = Square::of(rank, file);
                if self.test(sq) {
                    write!(f, " 1 ")?
                } else {
                    write!(f, " . ")?
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

// Operator overloads for ease of use
impl ops::BitAnd for Bitboard {
    type Output = Bitboard;

    fn bitand(self, rhs: Bitboard) -> Bitboard {
        self.and(rhs)
    }
}

impl ops::BitAndAssign for Bitboard {
    fn bitand_assign(&mut self, rhs: Bitboard) {
        *self = self.and(rhs);
    }
}

impl ops::BitOr for Bitboard {
    type Output = Bitboard;

    fn bitor(self, rhs: Bitboard) -> Bitboard {
        self.or(rhs)
    }
}

impl ops::BitOrAssign for Bitboard {
    fn bitor_assign(&mut self, rhs: Bitboard) {
        *self = self.or(rhs);
    }
}

impl ops::BitXor for Bitboard {
    type Output = Bitboard;

    fn bitxor(self, rhs: Bitboard) -> Bitboard {
        self.xor(rhs)
    }
}

impl ops::BitXorAssign for Bitboard {
    fn bitxor_assign(&mut self, rhs: Bitboard) {
        *self = self.xor(rhs);
    }
}

/// BitboardIterator is an iterator over squares that are set in a
/// given bitboard.
pub struct BitboardIterator {
    bits: u64,
}

impl BitboardIterator {
    fn new(bits: u64) -> BitboardIterator {
        BitboardIterator { bits: bits }
    }
}

impl Iterator for BitboardIterator {
    type Item = Square;

    fn next(&mut self) -> Option<Square> {
        if self.bits == 0 {
            return None;
        }

        let next = self.bits.trailing_zeros();
        self.bits &= self.bits - 1;
        Some(FromPrimitive::from_u32(next).unwrap())
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (0, Some(64))
    }
}

impl IntoIterator for Bitboard {
    type Item = Square;
    type IntoIter = BitboardIterator;

    fn into_iter(self) -> BitboardIterator {
        self.iter()
    }
}

fn initialize_knight_bitboards() {
    // for knights: https://chessprogramming.wikispaces.com/Knight+Pattern
    let mut knight_table = KNIGHT_ATTACKS.write();

    // collection of masks to ensure we don't go off the board
    let not_a_file = !FILE_MASKS[File::A as usize];
    let not_ab_file = !(FILE_MASKS[File::A as usize] | FILE_MASKS[File::B as usize]);
    let not_h_file = !FILE_MASKS[File::H as usize];
    let not_gh_file = !(FILE_MASKS[File::G as usize] | FILE_MASKS[File::H as usize]);
    for sq_idx in (Square::A1 as u64)..(Square::H8 as u64) {
        let mut board = Bitboard::none();
        let sq_bit = 1u64 << sq_idx;

        {
            let mut add_move = |bits: u64, mask: u64| {
                let target = FromPrimitive::from_u64(bits & mask);
                board.set(target.unwrap());
            };

            // the eight possible knight moves.
            add_move(sq_bit << 17, not_a_file);
            add_move(sq_bit << 10, not_ab_file);
            add_move(sq_bit >> 6, not_ab_file);
            add_move(sq_bit >> 15, not_a_file);
            add_move(sq_bit << 15, not_h_file);
            add_move(sq_bit << 6, not_gh_file);
            add_move(sq_bit << 10, not_gh_file);
            add_move(sq_bit << 17, not_h_file);
        }

        knight_table[sq_idx as usize] = board;
    }
}

fn initialize_pawn_bitboards() {
    // since the pawn is the only piece whose movement is influenced
    // by the side of the player owning the pawn, the pawn movement board
    // is composed of two boards - one for each side.
    let mut pawn_table = PAWN_ATTACKS.write();
    let not_a_file = !FILE_MASKS[File::A as usize];
    let not_h_file = !FILE_MASKS[File::H as usize];
    for sq_idx in (Square::A1 as u64)..(Square::H8 as u64) {
        let mut white_board = Bitboard::none();
        let mut black_board = Bitboard::none();
        let sq_bit = 1u64 << sq_idx;

        // pawns capture up and to the left and up and to the right.
        // by our compass rose, that corresponds to +/- 7 and +/- 9.
        let white_mask = ((sq_bit << 7) & not_h_file) | ((sq_bit << 9) & not_a_file);
        let black_mask = ((sq_bit >> 7) & not_a_file) | ((sq_bit >> 9) & not_h_file);

        if let Some(sq) = FromPrimitive::from_u64(white_mask) {
            white_board.set(sq);
        } else {
            panic!("does this happen?");
        }

        if let Some(sq) = FromPrimitive::from_u64(black_mask) {
            black_board.set(sq);
        } else {
            panic!("does this happen?");
        }

        pawn_table[Color::White as usize][sq_idx as usize] = white_board;
        pawn_table[Color::Black as usize][sq_idx as usize] = black_board;
    }
}

/// Initializes the global bitboards, which are lists of pre-computed moves
/// used by the engine to quickly query piece moves.
pub fn initialize() {
    initialize_pawn_bitboards();
    initialize_knight_bitboards();
}

/// Calculates the bitboard of attack squares for a given piece.
pub fn attacks(piece: Piece, square: Square, occupancy: Bitboard) -> Bitboard {
    match piece.kind {
        PieceKind::Pawn => {
            let pawn_attacks = PAWN_ATTACKS.read();
            pawn_attacks[piece.color as usize][square as usize]
        }
        PieceKind::Knight => {
            let knight_attacks = KNIGHT_ATTACKS.read();
            knight_attacks[square as usize]
        }
        PieceKind::Bishop => slides::bishop_attacks(square, occupancy),
        PieceKind::Rook => slides::rook_attacks(square, occupancy),
        PieceKind::Queen => slides::queen_attacks(square, occupancy),
        PieceKind::King => {
            let king_attacks = KING_ATTACKS.read();
            king_attacks[square as usize]
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn smoke_test() {
        let mut board = Bitboard::default();
        assert!(!board.test(Square::A1));

        board.set(Square::A1);
        assert!(board.test(Square::A1));
    }

    #[test]
    fn union() {
        let mut one = Bitboard::default();
        let mut two = Bitboard::default();
        one.set(Square::A2);
        two.set(Square::B2);

        assert!(!two.test(Square::A2));
        assert!(!one.test(Square::B2));

        let three = one | two;
        assert!(three.test(Square::A2));
        assert!(three.test(Square::B2));
    }

    #[test]
    fn intesection() {
        let mut one = Bitboard::default();
        let mut two = Bitboard::default();
        one.set(Square::A2);
        one.set(Square::B2);
        two.set(Square::A2);
        two.set(Square::C2);

        let three = one & two;
        assert!(three.test(Square::A2));
        assert!(!three.test(Square::B2));
        assert!(!three.test(Square::C2));
    }

    #[test]
    fn enumerating() {
        let mut one = Bitboard::default();
        one.set(Square::A2);
        one.set(Square::B2);

        let squares: Vec<_> = one.iter().collect();
        assert_eq!(2, squares.len());
        assert_eq!(Square::A2, squares[0]);
        assert_eq!(Square::B2, squares[1]);
    }

    #[test]
    fn empty_iter() {
        let one = Bitboard::default();
        let squares: Vec<_> = one.iter().collect();
        assert_eq!(0, squares.len());
    }

    #[test]
    fn rank() {
        fn rank_test(rank: Rank, on_rank: Square, off_rank: Square) {
            let mut one = Bitboard::none();
            one.set(on_rank);
            one.set(off_rank);

            let composite = one.rank(rank);
            assert!(composite.test(on_rank));
            assert!(!composite.test(off_rank));
        }

        rank_test(Rank::Rank8, Square::E8, Square::E7);
        rank_test(Rank::Rank7, Square::E7, Square::E6);
        rank_test(Rank::Rank6, Square::E6, Square::E7);
        rank_test(Rank::Rank5, Square::A5, Square::A8);
        rank_test(Rank::Rank4, Square::B4, Square::B5);
        rank_test(Rank::Rank3, Square::C3, Square::C4);
        rank_test(Rank::Rank2, Square::B2, Square::F8);
        rank_test(Rank::Rank1, Square::H1, Square::F2);
    }

    #[test]
    fn unset() {
        let mut board = Bitboard::none();
        board.set(Square::H2);
        assert!(board.test(Square::H2));
        board.unset(Square::H2);
        assert!(!board.test(Square::H2));
        assert!(board.count() == 0);
    }

    #[test]
    fn count() {
        let mut board = Bitboard::none();
        board.set(Square::A2);
        board.set(Square::B5);
        board.set(Square::H8);
        assert!(board.count() == 3);
    }
}
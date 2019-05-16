// Copyright 2017-2019 Sean Gillespie.
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
use num_traits::FromPrimitive;
use std::default::Default;
use std::fmt;
use std::iter::Iterator;
use std::ops;

use crate::types::{self, File, Rank, Square};

const RANK_MASKS: [u64; 8] = [
    0x0000_0000_0000_00FF,
    0x0000_0000_0000_FF00,
    0x0000_0000_00FF_0000,
    0x0000_0000_FF00_0000,
    0x0000_00FF_0000_0000,
    0x0000_FF00_0000_0000,
    0x00FF_0000_0000_0000,
    0xFF00_0000_0000_0000,
];

const FILE_MASKS: [u64; 8] = [
    0x0101_0101_0101_0101,
    0x0202_0202_0202_0202,
    0x0404_0404_0404_0404,
    0x0808_0808_0808_0808,
    0x1010_1010_1010_1010,
    0x2020_2020_2020_2020,
    0x4040_4040_4040_4040,
    0x8080_8080_8080_8080,
];

/// A Bitboard is a 64-bit integer which one bit represents one of the
/// eight squares on the board. Bitboards are used in a variety of scenarios
/// to represent the board itself and the pieces upon it.
#[derive(Copy, Clone)]
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
        Bitboard { bits }
    }

    /// Constructs a new bitboard with all bits set to one, representing
    /// a complete set.
    pub const fn all() -> Bitboard {
        Bitboard::from_bits(0xFFFF_FFFF_FFFF_FFFF)
    }

    /// Constructs a new bitboard with all bits zeroed, representing
    /// the empty set.
    pub const fn none() -> Bitboard {
        Bitboard::from_bits(0)
    }

    /// Tests whether or not a square is a member of this bitboard.
    pub const fn test(self, square: Square) -> bool {
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
    pub const fn and(self, other: Bitboard) -> Bitboard {
        Bitboard::from_bits(self.bits & other.bits)
    }

    /// Takes the bitwise or of two bitboards producing the set union
    /// of their contents.
    pub const fn or(self, other: Bitboard) -> Bitboard {
        Bitboard::from_bits(self.bits | other.bits)
    }

    /// Takes the bitwise exclusive or of two bitboards.
    pub const fn xor(self, other: Bitboard) -> Bitboard {
        Bitboard::from_bits(self.bits ^ other.bits)
    }

    /// Produces an iterator over the squares contained in this bitboard.
    pub fn iter(self) -> BitboardIterator {
        BitboardIterator::new(self.bits)
    }

    /// Produces a bitboard with the components of this bitboard that
    /// lie on the given rank.
    pub const fn rank(self, rank: Rank) -> Bitboard {
        self.and(Bitboard::from_bits(RANK_MASKS[rank as usize]))
    }

    /// Produces a bitboard with the components of this bitboard that
    /// lie on the given file.
    pub const fn file(self, file: File) -> Bitboard {
        self.and(Bitboard::from_bits(FILE_MASKS[file as usize]))
    }

    /// Retireves the raw bits associated with this bitboard.
    pub const fn bits(self) -> u64 {
        self.bits
    }

    /// Retrieves the number of squares contained in the set represented
    /// by this bitboard.
    pub const fn count(self) -> u32 {
        self.bits.count_ones()
    }

    /// Retrieves whether or not the set represented by this bitboard is
    /// the empty set.
    pub const fn empty(self) -> bool {
        self.bits == 0
    }

    /// Retrieves one piece in the set represented by this bitboard.
    pub fn first(self) -> Option<Square> {
        self.into_iter().next()
    }
}

impl fmt::Debug for Bitboard {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_tuple("Bitboard").field(&self.bits).finish()
    }
}

impl fmt::Display for Bitboard {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for &rank in types::RANKS.iter().rev() {
            for &file in &types::FILES {
                let sq = Square::of(rank, file);
                if self.test(sq) {
                    write!(f, " 1 ")?
                } else {
                    write!(f, " . ")?
                }
            }

            writeln!(f, "| {}", rank)?;
        }

        for _ in &types::FILES {
            write!(f, "---")?;
        }

        writeln!(f)?;
        for file in &types::FILES {
            write!(f, " {} ", file)?;
        }

        writeln!(f)?;
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
        BitboardIterator { bits }
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

pub const BB_RANK_1: Bitboard = Bitboard::from_bits(RANK_MASKS[0]);
pub const BB_RANK_2: Bitboard = Bitboard::from_bits(RANK_MASKS[1]);
pub const BB_RANK_3: Bitboard = Bitboard::from_bits(RANK_MASKS[2]);
pub const BB_RANK_4: Bitboard = Bitboard::from_bits(RANK_MASKS[3]);
pub const BB_RANK_5: Bitboard = Bitboard::from_bits(RANK_MASKS[4]);
pub const BB_RANK_6: Bitboard = Bitboard::from_bits(RANK_MASKS[5]);
pub const BB_RANK_7: Bitboard = Bitboard::from_bits(RANK_MASKS[6]);
pub const BB_RANK_8: Bitboard = Bitboard::from_bits(RANK_MASKS[7]);

pub const BB_FILE_A: Bitboard = Bitboard::from_bits(FILE_MASKS[0]);
pub const BB_FILE_B: Bitboard = Bitboard::from_bits(FILE_MASKS[1]);
pub const BB_FILE_C: Bitboard = Bitboard::from_bits(FILE_MASKS[2]);
pub const BB_FILE_D: Bitboard = Bitboard::from_bits(FILE_MASKS[3]);
pub const BB_FILE_E: Bitboard = Bitboard::from_bits(FILE_MASKS[4]);
pub const BB_FILE_F: Bitboard = Bitboard::from_bits(FILE_MASKS[5]);
pub const BB_FILE_G: Bitboard = Bitboard::from_bits(FILE_MASKS[6]);
pub const BB_FILE_H: Bitboard = Bitboard::from_bits(FILE_MASKS[7]);

pub const BB_FILE_AB: Bitboard = BB_FILE_A.or(BB_FILE_B);
pub const BB_FILE_GH: Bitboard = BB_FILE_G.or(BB_FILE_H);

pub const BB_RANK_12: Bitboard = BB_RANK_1.or(BB_RANK_2);
pub const BB_RANK_78: Bitboard = BB_RANK_7.or(BB_RANK_8);

pub const BB_RANKS: [Bitboard; 8] = [
    BB_RANK_1, BB_RANK_2, BB_RANK_3, BB_RANK_4, BB_RANK_5, BB_RANK_6, BB_RANK_7, BB_RANK_8,
];

pub const BB_FILES: [Bitboard; 8] = [
    BB_FILE_A, BB_FILE_B, BB_FILE_C, BB_FILE_D, BB_FILE_E, BB_FILE_F, BB_FILE_G, BB_FILE_H,
];

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

        rank_test(Rank::Eight, Square::E8, Square::E7);
        rank_test(Rank::Seven, Square::E7, Square::E6);
        rank_test(Rank::Six, Square::E6, Square::E7);
        rank_test(Rank::Five, Square::A5, Square::A8);
        rank_test(Rank::Four, Square::B4, Square::B5);
        rank_test(Rank::Three, Square::C3, Square::C4);
        rank_test(Rank::Two, Square::B2, Square::F8);
        rank_test(Rank::One, Square::H1, Square::F2);
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

// Copyright 2017-2019 Sean Gillespie.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.
use num_traits::{FromPrimitive, ToPrimitive};
use std::convert::TryFrom;
use std::fmt::{self, Display, Write};

use crate::attacks;
use crate::bitboard::Bitboard;

// TableIndex is a trait for all types that can serve as an index into a table.
// It is common to use these types as indices into tables, so this trait allows
// any type implementing To and FromPrimitive to be used as table indices.
pub trait TableIndex {
    fn as_index(self) -> usize;
    fn from_index(idx: usize) -> Self;
}

impl<T> TableIndex for T
where
    T: FromPrimitive + ToPrimitive,
{
    fn as_index(self) -> usize {
        self.to_u32().unwrap() as usize
    }

    fn from_index(idx: usize) -> T {
        <T as FromPrimitive>::from_u64(idx as u64).unwrap()
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, FromPrimitive, ToPrimitive)]
pub enum Square {
    A1,
    B1,
    C1,
    D1,
    E1,
    F1,
    G1,
    H1,
    A2,
    B2,
    C2,
    D2,
    E2,
    F2,
    G2,
    H2,
    A3,
    B3,
    C3,
    D3,
    E3,
    F3,
    G3,
    H3,
    A4,
    B4,
    C4,
    D4,
    E4,
    F4,
    G4,
    H4,
    A5,
    B5,
    C5,
    D5,
    E5,
    F5,
    G5,
    H5,
    A6,
    B6,
    C6,
    D6,
    E6,
    F6,
    G6,
    H6,
    A7,
    B7,
    C7,
    D7,
    E7,
    F7,
    G7,
    H7,
    A8,
    B8,
    C8,
    D8,
    E8,
    F8,
    G8,
    H8,
}

impl Square {
    pub fn of(rank: Rank, file: File) -> Square {
        let rank = rank.to_u32().unwrap();
        let file = file.to_u32().unwrap();
        FromPrimitive::from_u32(rank * 8 + file).unwrap()
    }

    pub fn rank(self) -> Rank {
        FromPrimitive::from_u32(self.to_u32().unwrap() >> 3).unwrap()
    }

    pub fn file(self) -> File {
        FromPrimitive::from_u32(self.to_u32().unwrap() & 7).unwrap()
    }

    pub fn plus(self, offset: i32) -> Square {
        let prim = self.to_i32().unwrap();
        FromPrimitive::from_i32(prim + offset).unwrap()
    }

    pub fn towards(self, dir: Direction) -> Square {
        self.plus(dir.as_vector())
    }
}

impl Display for Square {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}{}", self.file(), self.rank())
    }
}

pub static SQUARES: [Square; 64] = [
    Square::A1,
    Square::B1,
    Square::C1,
    Square::D1,
    Square::E1,
    Square::F1,
    Square::G1,
    Square::H1,
    Square::A2,
    Square::B2,
    Square::C2,
    Square::D2,
    Square::E2,
    Square::F2,
    Square::G2,
    Square::H2,
    Square::A3,
    Square::B3,
    Square::C3,
    Square::D3,
    Square::E3,
    Square::F3,
    Square::G3,
    Square::H3,
    Square::A4,
    Square::B4,
    Square::C4,
    Square::D4,
    Square::E4,
    Square::F4,
    Square::G4,
    Square::H4,
    Square::A5,
    Square::B5,
    Square::C5,
    Square::D5,
    Square::E5,
    Square::F5,
    Square::G5,
    Square::H5,
    Square::A6,
    Square::B6,
    Square::C6,
    Square::D6,
    Square::E6,
    Square::F6,
    Square::G6,
    Square::H6,
    Square::A7,
    Square::B7,
    Square::C7,
    Square::D7,
    Square::E7,
    Square::F7,
    Square::G7,
    Square::H7,
    Square::A8,
    Square::B8,
    Square::C8,
    Square::D8,
    Square::E8,
    Square::F8,
    Square::G8,
    Square::H8,
];

#[derive(Copy, Clone, Debug, PartialEq, Eq, FromPrimitive, ToPrimitive)]
pub enum Rank {
    One,
    Two,
    Three,
    Four,
    Five,
    Six,
    Seven,
    Eight,
}

impl Display for Rank {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let chr = match self {
            Rank::One => '1',
            Rank::Two => '2',
            Rank::Three => '3',
            Rank::Four => '4',
            Rank::Five => '5',
            Rank::Six => '6',
            Rank::Seven => '7',
            Rank::Eight => '8',
        };
        f.write_char(chr)
    }
}

impl TryFrom<char> for Rank {
    type Error = ();

    fn try_from(value: char) -> Result<Self, Self::Error> {
        let res = match value {
            '1' => Rank::One,
            '2' => Rank::Two,
            '3' => Rank::Three,
            '4' => Rank::Four,
            '5' => Rank::Five,
            '6' => Rank::Six,
            '7' => Rank::Seven,
            '8' => Rank::Eight,
            _ => return Err(()),
        };
        Ok(res)
    }
}

pub static RANKS: [Rank; 8] = [
    Rank::One,
    Rank::Two,
    Rank::Three,
    Rank::Four,
    Rank::Five,
    Rank::Six,
    Rank::Seven,
    Rank::Eight,
];

#[derive(Copy, Clone, Debug, PartialEq, Eq, FromPrimitive, ToPrimitive)]
pub enum File {
    A,
    B,
    C,
    D,
    E,
    F,
    G,
    H,
}

impl Display for File {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let chr = match self {
            File::A => 'a',
            File::B => 'b',
            File::C => 'c',
            File::D => 'd',
            File::E => 'e',
            File::F => 'f',
            File::G => 'g',
            File::H => 'h',
        };
        f.write_char(chr)
    }
}

impl TryFrom<char> for File {
    type Error = ();

    fn try_from(value: char) -> Result<Self, Self::Error> {
        let res = match value {
            'a' => File::A,
            'b' => File::B,
            'c' => File::C,
            'd' => File::D,
            'e' => File::E,
            'f' => File::F,
            'g' => File::G,
            'h' => File::H,
            _ => return Err(()),
        };
        Ok(res)
    }
}

pub static FILES: [File; 8] = [
    File::A,
    File::B,
    File::C,
    File::D,
    File::E,
    File::F,
    File::G,
    File::H,
];

#[derive(Copy, Clone, Debug, PartialEq, Eq, FromPrimitive, ToPrimitive)]
pub enum Color {
    White,
    Black,
}

impl Color {
    pub fn toggle(self) -> Color {
        match self {
            Color::White => Color::Black,
            Color::Black => Color::White,
        }
    }
}

impl Display for Color {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let chr = match self {
            Color::White => 'w',
            Color::Black => 'b',
        };
        f.write_char(chr)
    }
}

pub static COLORS: [Color; 2] = [Color::White, Color::Black];

#[derive(Copy, Clone, Debug, PartialEq, Eq, FromPrimitive, ToPrimitive)]
pub enum PieceKind {
    Pawn,
    Knight,
    Bishop,
    Rook,
    Queen,
    King,
}

impl Display for PieceKind {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let chr = match self {
            PieceKind::Pawn => 'p',
            PieceKind::Knight => 'n',
            PieceKind::Bishop => 'b',
            PieceKind::Rook => 'r',
            PieceKind::Queen => 'q',
            PieceKind::King => 'k',
        };
        f.write_char(chr)
    }
}

pub static PIECE_KINDS: [PieceKind; 6] = [
    PieceKind::Pawn,
    PieceKind::Knight,
    PieceKind::Bishop,
    PieceKind::Rook,
    PieceKind::Queen,
    PieceKind::King,
];

#[derive(Copy, Clone, Debug, PartialEq, Eq, FromPrimitive, ToPrimitive)]
pub enum Direction {
    North,
    NorthEast,
    East,
    SouthEast,
    South,
    SouthWest,
    West,
    NorthWest,
}

impl Direction {
    pub fn as_vector(self) -> i32 {
        match self {
            Direction::North => 8,
            Direction::NorthEast => 9,
            Direction::East => 1,
            Direction::SouthEast => -7,
            Direction::South => -8,
            Direction::SouthWest => -9,
            Direction::West => -1,
            Direction::NorthWest => 7,
        }
    }
}

bitflags! {
    pub struct CastleStatus: u8 {
        const NONE = 0;
        const WHITE_KINGSIDE = 0b0000_0001;
        const WHITE_QUEENSIDE =0b0000_0010;
        const WHITE = Self::WHITE_KINGSIDE.bits | Self::WHITE_QUEENSIDE.bits;
        const BLACK_KINGSIDE = 0b0000_0100;
        const BLACK_QUEENSIDE = 0b0000_1000;
        const BLACK = Self::BLACK_KINGSIDE.bits | Self::BLACK_QUEENSIDE.bits;
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Piece {
    pub kind: PieceKind,
    pub color: Color,
}

impl Piece {
    pub fn new(kind: PieceKind, color: Color) -> Piece {
        Piece { kind, color }
    }

    pub fn attacks(&self, sq: Square, occupancy: Bitboard) -> Bitboard {
        match self.kind {
            PieceKind::Pawn => attacks::pawn_attacks(sq, self.color),
            PieceKind::Knight => attacks::knight_attacks(sq),
            PieceKind::Bishop => attacks::bishop_attacks(sq, occupancy),
            PieceKind::Rook => attacks::rook_attacks(sq, occupancy),
            PieceKind::Queen => attacks::queen_attacks(sq, occupancy),
            PieceKind::King => attacks::king_attacks(sq),
        }
    }

    pub fn is_sliding(&self) -> bool {
        match self.kind {
            PieceKind::Pawn | PieceKind::Knight | PieceKind::King => false,
            _ => true,
        }
    }
}

impl TryFrom<char> for Piece {
    type Error = ();

    fn try_from(c: char) -> Result<Self, Self::Error> {
        let res = match c {
            'P' => Piece::new(PieceKind::Pawn, Color::White),
            'N' => Piece::new(PieceKind::Knight, Color::White),
            'B' => Piece::new(PieceKind::Bishop, Color::White),
            'R' => Piece::new(PieceKind::Rook, Color::White),
            'Q' => Piece::new(PieceKind::Queen, Color::White),
            'K' => Piece::new(PieceKind::King, Color::White),
            'p' => Piece::new(PieceKind::Pawn, Color::Black),
            'n' => Piece::new(PieceKind::Knight, Color::Black),
            'b' => Piece::new(PieceKind::Bishop, Color::Black),
            'r' => Piece::new(PieceKind::Rook, Color::Black),
            'q' => Piece::new(PieceKind::Queen, Color::Black),
            'k' => Piece::new(PieceKind::King, Color::Black),
            _ => return Err(()),
        };
        Ok(res)
    }
}

impl Display for Piece {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let chr = match self {
            Piece {
                kind: PieceKind::Pawn,
                color: Color::White,
            } => 'P',
            Piece {
                kind: PieceKind::Knight,
                color: Color::White,
            } => 'N',
            Piece {
                kind: PieceKind::Bishop,
                color: Color::White,
            } => 'B',
            Piece {
                kind: PieceKind::Rook,
                color: Color::White,
            } => 'R',
            Piece {
                kind: PieceKind::Queen,
                color: Color::White,
            } => 'Q',
            Piece {
                kind: PieceKind::King,
                color: Color::White,
            } => 'K',
            Piece {
                kind: PieceKind::Pawn,
                color: Color::Black,
            } => 'p',
            Piece {
                kind: PieceKind::Knight,
                color: Color::Black,
            } => 'n',
            Piece {
                kind: PieceKind::Bishop,
                color: Color::Black,
            } => 'b',
            Piece {
                kind: PieceKind::Rook,
                color: Color::Black,
            } => 'r',
            Piece {
                kind: PieceKind::Queen,
                color: Color::Black,
            } => 'q',
            Piece {
                kind: PieceKind::King,
                color: Color::Black,
            } => 'k',
        };

        f.write_char(chr)
    }
}

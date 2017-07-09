// Copyright 2017 Sean Gillespie.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.
use num_traits::FromPrimitive;
use std::fmt;

/// Little-endian rank-file (LERF) mapping from squares to bits in a bitboard.
/// Directions are computed by adding constants to a square.
#[repr(u8)]
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Square {
   A1 = 0,  B1 = 1,  C1 = 2,  D1 = 3,  E1 = 4,  F1 = 5,  G1 = 6,  H1 = 7,
   A2 = 8,  B2 = 9,  C2 = 10, D2 = 11, E2 = 12, F2 = 13, G2 = 14, H2 = 15,
   A3 = 16, B3 = 17, C3 = 18, D3 = 19, E3 = 20, F3 = 21, G3 = 22, H3 = 23,
   A4 = 24, B4 = 25, C4 = 26, D4 = 27, E4 = 28, F4 = 29, G4 = 30, H4 = 31,
   A5 = 32, B5 = 33, C5 = 34, D5 = 35, E5 = 36, F5 = 37, G5 = 38, H5 = 39,
   A6 = 40, B6 = 41, C6 = 42, D6 = 43, E6 = 44, F6 = 45, G6 = 46, H6 = 47,
   A7 = 48, B7 = 49, C7 = 50, D7 = 51, E7 = 52, F7 = 53, G7 = 54, H7 = 55,
   A8 = 56, B8 = 57, C8 = 58, D8 = 59, E8 = 60, F8 = 61, G8 = 62, H8 = 63
}

impl FromPrimitive for Square {
    fn from_i64(n: i64) -> Option<Square> {
        <Square as FromPrimitive>::from_u64(n as u64)
    }

    fn from_u64(n: u64) -> Option<Square> {
        use types::Square::*;
        let result = match n {
            0 => A1,  1 => B1,  2 => C1,  3 => D1,  4 => E1,  5 => F1,  6 => G1,  7  => H1,
            8 => A2,  9 => B2,  10 => C2, 11 => D2, 12 => E2, 13 => F2, 14 => G2, 15 => H2,
            16 => A3, 17 => B3, 18 => C3, 19 => D3, 20 => E3, 21 => F3, 22 => G3, 23 => H3,
            24 => A4, 25 => B4, 26 => C4, 27 => D4, 28 => E4, 29 => F4, 30 => G4, 31 => H4, 
            32 => A5, 33 => B5, 34 => C5, 35 => D5, 36 => E5, 37 => F5, 38 => G5, 39 => H5, 
            40 => A6, 41 => B6, 42 => C6, 43 => D6, 44 => E6, 45 => F6, 46 => G6, 47 => H6, 
            48 => A7, 49 => B7, 50 => C7, 51 => D7, 52 => E7, 53 => F7, 54 => G7, 55 => H7, 
            56 => A8, 57 => B8, 58 => C8, 59 => D8, 60 => E8, 61 => F8, 62 => G8, 63 => H8,
            _ => return None
        };

        Some(result)
    }
}

impl Square {
    /// Constructs a Square from a Rank and File.
    pub fn of(rank: Rank, file: File) -> Square {
        // the relationship between rank, file, and square:
        //    rank = square % 8
        //    file = square / 8
        //    square = rank * 8 + file
        FromPrimitive::from_u32((rank as u32) * 8 + (file as u32)).unwrap()
    }

    pub fn rank(self) -> Rank {
        FromPrimitive::from_u64((self as u64) >> 3).unwrap()
    }

    pub fn file(self) -> File {
        FromPrimitive::from_u64((self as u64) & 7).unwrap()
    }
}

impl fmt::Display for Square {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(f, "{}{}", self.file(), self.rank())
    }
}

/// Enum representing a rank on the chessboard.
#[repr(u8)]
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Rank {
    Rank1 = 0, Rank2 = 1, Rank3 = 2, Rank4 = 3,
    Rank5 = 4, Rank6 = 5, Rank7 = 6, Rank8 = 7
}

impl Rank {
    pub fn from_char(c: char) -> Option<Rank> {
        let result = match c {
            '1' => Rank::Rank1,
            '2' => Rank::Rank2,
            '3' => Rank::Rank3,
            '4' => Rank::Rank4,
            '5' => Rank::Rank5,
            '6' => Rank::Rank6,
            '7' => Rank::Rank7,
            '8' => Rank::Rank8,
            _ => return None
        };

        Some(result)
    }
}

impl FromPrimitive for Rank {
    fn from_i64(x: i64) -> Option<Rank> {
        <Rank as FromPrimitive>::from_u64(x as u64)
    }

    fn from_u64(x: u64) -> Option<Rank> {
        let result = match x {
            0 => Rank::Rank1,
            1 => Rank::Rank2,
            2 => Rank::Rank3,
            3 => Rank::Rank4,
            4 => Rank::Rank5,
            5 => Rank::Rank6,
            6 => Rank::Rank7,
            7 => Rank::Rank8,
            _ => return None
        };

        Some(result)
    }
}

impl fmt::Display for Rank {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        let chr = match *self {
            Rank::Rank1 => '1',
            Rank::Rank2 => '2',
            Rank::Rank3 => '3',
            Rank::Rank4 => '4',
            Rank::Rank5 => '5',
            Rank::Rank6 => '6',
            Rank::Rank7 => '7',
            Rank::Rank8 => '8'
        };

        write!(f, "{}", chr)
    }
}

/// Enum representing a file on the chessboard.
#[repr(u8)]
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum File {
    A = 0, B = 1, C = 2, D = 3, E = 4, F = 5, G = 6, H = 7
}

impl File {
    pub fn from_char(c: char) -> Option<File> {
        let result = match c {
            'a' => File::A,
            'b' => File::B,
            'c' => File::C,
            'd' => File::D,
            'e' => File::E,
            'f' => File::F,
            'g' => File::G,
            'h' => File::H,
            _ => return None
        };

        Some(result)
    }
}

impl FromPrimitive for File {
    fn from_i64(x: i64) -> Option<File> {
        <File as FromPrimitive>::from_u64(x as u64)
    }

    fn from_u64(x: u64) -> Option<File> {
        let result = match x {
            0 => File::A,
            1 => File::B,
            2 => File::C,
            3 => File::D,
            4 => File::E,
            5 => File::F,
            6 => File::G,
            7 => File::H,
            _ => return None
        };

        Some(result)
    }
}

impl fmt::Display for File {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        let chr = match *self {
            File::A => 'a',
            File::B => 'b',
            File::C => 'c',
            File::D => 'd',
            File::E => 'e',
            File::F => 'f',
            File::G => 'g',
            File::H => 'h'
        };

        write!(f, "{}", chr)
    }
}

/// Enum representing the player colors.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum Color {
    White,
    Black
}

impl Color {
    pub fn toggle(self) -> Color {
        match self {
            Color::White => Color::Black,
            Color::Black => Color::White
        }
    }
}

/// The kinds of chess pieces.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum PieceKind {
    Pawn    = 0,
    Knight  = 1,
    Bishop  = 2,
    Rook    = 3,
    Queen   = 4,
    King    = 5
}

impl FromPrimitive for PieceKind {
    fn from_i64(x: i64) -> Option<Self> {
        <PieceKind as FromPrimitive>::from_u64(x as u64)
    }

    fn from_u64(x: u64) -> Option<Self> {
        let val = match x {
            0 => PieceKind::Pawn,
            1 => PieceKind::Knight,
            2 => PieceKind::Bishop,
            3 => PieceKind::Rook,
            4 => PieceKind::Queen,
            5 => PieceKind::King,
            _ => return None
        };

        Some(val)
    }
}

/// A Piece is a collection of a PieceKind and a Color.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Piece {
    pub kind: PieceKind,
    pub color: Color
}

impl Piece {
    /// Constructs a new Ppiece from a PieceKind and a Color.
    pub fn new(kind: PieceKind, color: Color) -> Piece {
        Piece {
            kind: kind,
            color: color
        }
    }
}

#[repr(u8)]
#[derive(Debug, Copy, Clone)]
pub enum Direction {
    North,
    NorthEast,
    East,
    SouthEast,
    South,
    SouthWest,
    West,
    NorthWest
}

impl Direction {
    /// Turns this direction into a "vector" that can be added to a square
    /// to produce a new square in the given direction.
    pub fn as_vector(self) -> i8 {
        match self {
            Direction::North => 8,
            Direction::NorthEast => 9,
            Direction::East => 1,
            Direction::SouthEast => -7,
            Direction::South => -8,
            Direction::SouthWest => -9,
            Direction::West => -1,
            Direction::NorthWest => 7
        }
    }
}

bitflags! {
    pub struct CastleStatus : u8 {
        const CASTLE_NONE = 0x00;
        const WHITE_O_O = 0x01;
        const WHITE_O_O_O = 0x02;
        const WHITE_MASK = 0x03;
        const BLACK_O_O = 0x04;
        const BLACK_O_O_O = 0x08;
        const BLACK_MASK = 0x0C;
        const CASTLE_ALL = 0x0F;
    }
}
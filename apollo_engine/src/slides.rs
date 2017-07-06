// Copyright 2017 Sean Gillespie. See the COPYRIGHT
// file at the top-level directory of this distribution.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! The `slides` module encapsules the logic of sliding move generation.
//! For the purposes of speed, sliding moves are precomputed and stored in a
//! table which is then used by the move generator when generating moves or
//! by the position evaluator when determining whether or not a king is in check.
//!
//! This module currently implements the "classic method" of move generation,
//! which precomputes sliding rays of attack for every piece on the board and
//! every direction. Movesets for queens, rooks, and bishops can be constructed
//! by taking the union of move rays in the directions legal for that piece.
//!
//! All of the functions in this module consider the first blocking piece along
//! a ray to be a legal move, which it is if the first blocking piece is an
//! enemy piece. It is the responsibility of callers of this function to determine
//! whether or not the blocking piece is an enemy piece.
use num_traits::FromPrimitive;
use types::{Square, Direction, Rank, File};
use bitboard::Bitboard;
use parking_lot::RwLock;

static RAY_TABLE: RwLock<[[Bitboard; 8]; 65]> = RwLock::new([[Bitboard::none(); 8]; 65]);

fn ray_attacks(square: Square, occupancy: Bitboard, dir: Direction) -> Bitboard {
    let ray_table = RAY_TABLE.read();
    let attacks = ray_table[square as usize][dir as usize];
    let blocker = (attacks & occupancy).bits();
    let blocking_square = blocker.trailing_zeros();
    let blocking_ray = ray_table[blocking_square as usize][dir as usize];
    attacks ^ blocking_ray
}

/// Returns the a bitboard of legal diagonal attacks for a piece at the given
/// square and with the given board occupancy.
pub fn diagonal_attacks(square: Square, occupancy: Bitboard) -> Bitboard {
    ray_attacks(square, occupancy, Direction::NorthWest) |
    ray_attacks(square, occupancy, Direction::SouthEast)
}

/// Returns the bitboard of legal antidiagonal attacks for a piece at the given
/// square and with the given board occupancy.
pub fn antidiagonal_attacks(square: Square, occupancy: Bitboard) -> Bitboard {
    ray_attacks(square, occupancy, Direction::NorthEast) |
    ray_attacks(square, occupancy, Direction::SouthWest)
}

/// Returns the bitboard of legal file attacks for a piece at the given square
/// and with the given board occupancy.
pub fn file_attacks(square: Square, occupancy: Bitboard) -> Bitboard {
    ray_attacks(square, occupancy, Direction::North) |
    ray_attacks(square, occupancy, Direction::South)
}

/// Returns the bitboard of legal rank attacks for a piece at the given square
/// and with the given board occupancy.
pub fn rank_attacks(square: Square, occupancy: Bitboard) -> Bitboard {
    ray_attacks(square, occupancy, Direction::East) |
    ray_attacks(square, occupancy, Direction::West)
}

/// Returns the bitboard of legal bishop moves for a piece at the given square
/// and with the given board occupancy.
pub fn bishop_attacks(square: Square, occupancy: Bitboard) -> Bitboard {
    diagonal_attacks(square, occupancy) | antidiagonal_attacks(square, occupancy)
}

/// Returns the bitboard of legal rook moves for a piece at the given square
/// and with the given board occupancy.
pub fn rook_attacks(square: Square, occupancy: Bitboard) -> Bitboard {
    file_attacks(square, occupancy) | rank_attacks(square, occupancy)
}

/// Returns the bitboard of legal queen moves for a piece at the given square
/// and with the given board occupancy.
pub fn queen_attacks(square: Square, occupancy: Bitboard) -> Bitboard {
    bishop_attacks(square, occupancy) | rook_attacks(square, occupancy)
}

/// Initializes all of the global precomputed state required for efficient
/// run-time lookups of sliding moves.
pub fn initialize() {
    // the idea here is to generate rays in every direction for every square
    // on the board, to be used by the above methods.
    let mut ray_table = RAY_TABLE.write();
    for sq in (Square::A1 as usize)...(Square::H8 as usize) {
        let mut populate_direction = |direction: Direction, edge: Bitboard| {
            let mut cursor: Square = FromPrimitive::from_u64(sq as u64).unwrap();
            if edge.test(cursor) {
                // nothing to do here, there are no legal moves on this ray
                // from this square.
                return;
            }

            // starting at the given square, cast a ray in the given direction
            // and add all bits to the ray mask.
            let entry = &mut ray_table[sq][direction as usize];
            loop {
                cursor = FromPrimitive::from_i8(cursor as i8 + direction.as_vector()).unwrap();
                entry.set(cursor);

                // did we reach the end of the board? if so, stop.
                if edge.test(cursor) {
                    break;
                }
            }
        };

        let rank8 = Bitboard::all().rank(Rank::Rank8);
        let rank1 = Bitboard::all().rank(Rank::Rank1);
        let filea = Bitboard::all().file(File::A);
        let fileh = Bitboard::all().file(File::H);
        populate_direction(Direction::North, rank8);
        populate_direction(Direction::NorthEast, rank8 | fileh);
        populate_direction(Direction::East, fileh);
        populate_direction(Direction::SouthEast, rank1 | fileh);
        populate_direction(Direction::South, rank1);
        populate_direction(Direction::SouthWest, rank1 | filea);
        populate_direction(Direction::West, filea);
        populate_direction(Direction::NorthWest, rank8 | filea);
    }
}

#[cfg(test)]
mod tests {
    use types::{Square, Direction};
    use bitboard::Bitboard;
    use test::{self, Bencher};

    #[test]
    fn center_rook() {
        super::initialize();

        let square = Square::D4;
        let moves = super::rook_attacks(square, Bitboard::none());
        println!("moves: ");
        println!("{}", moves);
        assert!(moves.test(Square::D5));
        assert!(moves.test(Square::D6));
        assert!(moves.test(Square::D7));
        assert!(moves.test(Square::D8));
        assert!(moves.test(Square::D3));
        assert!(moves.test(Square::D2));
        assert!(moves.test(Square::D1));
        assert!(moves.test(Square::E4));
        assert!(moves.test(Square::F4));
        assert!(moves.test(Square::G4));
        assert!(moves.test(Square::H4));
        assert!(moves.test(Square::C4));
        assert!(moves.test(Square::B4));
        assert!(moves.test(Square::A4));
    }

    #[test]
    fn center_rook_with_occupancy() {
        super::initialize();

        let square = Square::D4;
        let occupied_square = Square::F4;

        let mut occupancy = Bitboard::none();
        occupancy.set(occupied_square);
 
        let moves = super::rook_attacks(square, occupancy);
        println!("moves: ");
        println!("{}", moves);
        assert_eq!(12, moves.count());
        assert!(moves.test(Square::D5));
        assert!(moves.test(Square::D6));
        assert!(moves.test(Square::D7));
        assert!(moves.test(Square::D8));
        assert!(moves.test(Square::D3));
        assert!(moves.test(Square::D2));
        assert!(moves.test(Square::D1));
        assert!(moves.test(Square::E4));
        assert!(moves.test(Square::C4));
        assert!(moves.test(Square::B4));
        assert!(moves.test(Square::A4));
    }

    #[test]
    fn edge_rook() {
        super::initialize();
        let square = Square::A1;
        let moves = super::rook_attacks(square, Bitboard::none());
        println!("moves: ");
        println!("{}", moves);
        assert_eq!(14, moves.count());
        assert!(moves.test(Square::B1));
        assert!(moves.test(Square::C1));
        assert!(moves.test(Square::D1));
        assert!(moves.test(Square::E1));
        assert!(moves.test(Square::F1));
        assert!(moves.test(Square::G1));
        assert!(moves.test(Square::H1));
        assert!(moves.test(Square::A3));
        assert!(moves.test(Square::A4));
        assert!(moves.test(Square::A5));
        assert!(moves.test(Square::A6));
        assert!(moves.test(Square::A7));
        assert!(moves.test(Square::A8));
    }

    #[test]
    fn edge_rook_with_occupancy() {
        super::initialize();
        let square = Square::A1;
        let mut occupancy = Bitboard::none();
        occupancy.set(Square::A2);
        occupancy.set(Square::B1);
        let moves = super::rook_attacks(square, occupancy);
        println!("moves: ");
        println!("{}", moves);
        assert_eq!(2, moves.count());
        assert!(moves.test(Square::A2));
        assert!(moves.test(Square::B1));
    }

    #[test]
    fn center_bishop() {
        super::initialize();
        let square = Square::E4;
        let moves = super::bishop_attacks(square, Bitboard::none());
        println!("moves: ");
        println!("{}", moves);
        assert_eq!(13, moves.count());
        assert!(moves.test(Square::F5));
        assert!(moves.test(Square::G6));
        assert!(moves.test(Square::H7));
        assert!(moves.test(Square::F3));
        assert!(moves.test(Square::G2));
        assert!(moves.test(Square::H1));
        assert!(moves.test(Square::D5));
        assert!(moves.test(Square::C6));
        assert!(moves.test(Square::B7));
        assert!(moves.test(Square::A8));
        assert!(moves.test(Square::D3));
        assert!(moves.test(Square::C2));
        assert!(moves.test(Square::B1));
    }

    #[bench]
    fn single_ray_bench(b: &mut test::Bencher) {
        b.iter(|| {
            let square = test::black_box(Square::E4);
            let dir = test::black_box(Direction::SouthWest);
            super::ray_attacks(square, Bitboard::none(), dir);
        });
    }

    #[bench]
    fn center_rook_bench(b: &mut Bencher) {
        b.iter(|| {
            let square = test::black_box(Square::E4);
            super::rook_attacks(square, Bitboard::none())
        });
    }

    #[bench]
    fn center_bishop_bench(b: &mut Bencher) {
        b.iter(|| {
            let square = test::black_box(Square::E4);
            super::bishop_attacks(square, Bitboard::none())
        });
    }

    #[bench]
    fn center_queen_bench(b: &mut Bencher) {
        b.iter(|| {
            let square = test::black_box(Square::E4);
            super::queen_attacks(square, Bitboard::none())
        });
    }
}
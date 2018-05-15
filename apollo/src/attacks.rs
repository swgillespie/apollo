// Copyright 2017 Sean Gillespie.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! The `attacks` module encapsules the logic of attack move generation.
//! For the purposes of speed, attack moves are precomputed and stored in a
//! table which is then used by the move generator when generating moves or
//! by the position evaluator when determining whether or not a king is in check.
//!
//! This module currently implements the "classic method" of move generation,
//! which precomputes sliding rays of attack for every sliding piece on the
//! board and every direction. Movesets for queens, rooks, and bishops can be
//! constructed by taking the union of move rays in legal directions.
//! Movesets for kings, pawns, and knights do not need to consider blocking
//! pieces.
//!
//! All of the sliding functions in this module consider the first blocking
//! piece along a ray to be a legal move, which it is if the first blocking
//! piece is an enemy piece. It is the responsibility of callers of this
//! function to determine whether or not the blocking piece is an enemy piece.
use num_traits::FromPrimitive;
use types::{Square, Direction, Rank, File, Color};
use bitboard::Bitboard;

static mut RAY_TABLE: [[Bitboard; 8]; 65] = [[Bitboard::none(); 8]; 65];
static mut PAWN_TABLE: [[Bitboard; 2]; 64] = [[Bitboard::none(); 2]; 64];
static mut KNIGHT_TABLE: [Bitboard; 64] = [Bitboard::none(); 64];
static mut KING_TABLE: [Bitboard; 64] = [Bitboard::none(); 64];

// a ray is "positive" if the ray vector is positive, otherwise a ray is
// "negative". if a ray is negative, we need to use leading zeros intead of
// trailing zeros in order to find the blocking piece.
fn positive_ray_attacks(square: Square, occupancy: Bitboard, dir: Direction) -> Bitboard {
    debug_assert!(dir.as_vector() > 0);
    let ray_table = unsafe { &RAY_TABLE };
    let attacks = ray_table[square as usize][dir as usize];
    let blocker = (attacks & occupancy).bits();
    let blocking_square = blocker.trailing_zeros();
    let blocking_ray = ray_table[blocking_square as usize][dir as usize];
    attacks ^ blocking_ray
}

fn negative_ray_attacks(square: Square, occupancy: Bitboard, dir: Direction) -> Bitboard {
    debug_assert!(dir.as_vector() < 0);
    let ray_table = unsafe { &RAY_TABLE };
    let attacks = ray_table[square as usize][dir as usize];
    let blocker = (attacks & occupancy).bits();

    // this can be done branchless, but on x86 rustc will use cmov for this
    // pattern, which works fine for us here since we're just trying to avoid
    // branch mispredicts.
    let blocking_square = (64 - blocker.leading_zeros()).checked_sub(1).unwrap_or(64);
    let blocking_ray = ray_table[blocking_square as usize][dir as usize];
    attacks ^ blocking_ray
}

/// Returns the a bitboard of legal diagonal attacks for a piece at the given
/// square and with the given board occupancy.
pub fn diagonal_attacks(square: Square, occupancy: Bitboard) -> Bitboard {
    positive_ray_attacks(square, occupancy, Direction::NorthWest) |
    negative_ray_attacks(square, occupancy, Direction::SouthEast)
}

/// Returns the bitboard of legal antidiagonal attacks for a piece at the given
/// square and with the given board occupancy.
pub fn antidiagonal_attacks(square: Square, occupancy: Bitboard) -> Bitboard {
    positive_ray_attacks(square, occupancy, Direction::NorthEast) |
    negative_ray_attacks(square, occupancy, Direction::SouthWest)
}

/// Returns the bitboard of legal file attacks for a piece at the given square
/// and with the given board occupancy.
pub fn file_attacks(square: Square, occupancy: Bitboard) -> Bitboard {
    positive_ray_attacks(square, occupancy, Direction::North) |
    negative_ray_attacks(square, occupancy, Direction::South)
}

/// Returns the bitboard of legal rank attacks for a piece at the given square
/// and with the given board occupancy.
pub fn rank_attacks(square: Square, occupancy: Bitboard) -> Bitboard {
    positive_ray_attacks(square, occupancy, Direction::East) |
    negative_ray_attacks(square, occupancy, Direction::West)
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

/// Returns the bitboard of legal knight moves for a piece at the given square.
pub fn knight_attacks(square: Square) -> Bitboard {
    let knight_table = unsafe { &KNIGHT_TABLE };
    knight_table[square as usize]
}

/// Returns the bitboard of legal pawn moves for a pawn at the given square
/// and with the given color.
pub fn pawn_attacks(square: Square, side: Color) -> Bitboard {
    let pawn_table = unsafe { &PAWN_TABLE };
    pawn_table[square as usize][side as usize]
}

/// Returns the bitboard of legal king moves for the given square.
pub fn king_attacks(square: Square) -> Bitboard {
    let king_table = unsafe { &KING_TABLE };
    king_table[square as usize]
}

fn initialize_knights() {
    let knight_table = unsafe { &mut KNIGHT_TABLE };
    let a_file = Bitboard::all().file(File::A);
    let b_file = Bitboard::all().file(File::B);
    let g_file = Bitboard::all().file(File::G);
    let h_file = Bitboard::all().file(File::H);
    let rank_1 = Bitboard::all().rank(Rank::Rank1);
    let rank_2 = Bitboard::all().rank(Rank::Rank2);
    let rank_7 = Bitboard::all().rank(Rank::Rank7);
    let rank_8 = Bitboard::all().rank(Rank::Rank8);
    for sq_idx in (Square::A1 as u64)..=(Square::H8 as u64) {
        let sq = FromPrimitive::from_u64(sq_idx).unwrap();
        let mut board = Bitboard::none();

        // north-north-west
        if !a_file.test(sq) && !(rank_7 | rank_8).test(sq) {
            let target = FromPrimitive::from_u64(sq_idx + 15).unwrap();
            board.set(target);
        }

        // north-north-east
        if !h_file.test(sq) && !(rank_7 | rank_8).test(sq) {
            let target = FromPrimitive::from_u64(sq_idx + 17).unwrap();
            board.set(target);
        }

        // north-east-east
        if !(g_file | h_file).test(sq) && !rank_8.test(sq) {
            let target = FromPrimitive::from_u64(sq_idx + 10).unwrap();
            board.set(target);
        }

        // south-east-east
        if !(g_file | h_file).test(sq) && !rank_1.test(sq) {
            let target = FromPrimitive::from_u64(sq_idx - 6).unwrap();
            board.set(target);
        }

        // south-south-east
        if !h_file.test(sq) && !(rank_2 | rank_1).test(sq) {
            let target = FromPrimitive::from_u64(sq_idx - 15).unwrap();
            board.set(target);
        }

        // south-south-west
        if !a_file.test(sq) && !(rank_2 | rank_1).test(sq) {
            let target = FromPrimitive::from_u64(sq_idx - 17).unwrap();
            board.set(target);
        }

        // south-west-west
        if !(a_file | b_file).test(sq) && !rank_1.test(sq) {
            let target = FromPrimitive::from_u64(sq_idx - 10).unwrap();
            board.set(target);
        }

        // north-west-west
        if !(a_file | b_file).test(sq) && !rank_8.test(sq) {
            let target = FromPrimitive::from_u64(sq_idx + 6).unwrap();
            board.set(target);
        }

        knight_table[sq_idx as usize] = board;
    }
}

fn initialize_pawns() {
    let pawn_table = unsafe { &mut PAWN_TABLE };
    let a_file = Bitboard::all().file(File::A);
    let h_file = Bitboard::all().file(File::H);
    let rank_1 = Bitboard::all().rank(Rank::Rank1);
    let rank_8 = Bitboard::all().rank(Rank::Rank8);
    for sq_idx in (Square::A1 as u64)..=(Square::H8 as u64) {
        let sq = FromPrimitive::from_u64(sq_idx).unwrap();
        for color in (Color::White as usize)..=(Color::Black as usize) {
            let mut board = Bitboard::none();
            let (promo_rank, up_left, up_right) = if color == (Color::White as usize) {
                (rank_8, 7i64, 9i64)
            } else {
                (rank_1, -9, -7)
            };

            if promo_rank.test(sq) {
                // no legal moves for this particular pawn. it's generally
                // impossible for pawns to be on the promotion rank anyway
                // since they should be getting promoted.
                continue;
            }

            if !a_file.test(sq) {
                let target = FromPrimitive::from_i64(sq_idx as i64 + up_left).unwrap();
                board.set(target);
            }

            if !h_file.test(sq) {
                let target = FromPrimitive::from_i64(sq_idx as i64 + up_right).unwrap();
                board.set(target);
            }

            pawn_table[sq_idx as usize][color as usize] = board;
        }
    }
}

fn initialize_kings() {
    let king_table = unsafe { &mut KING_TABLE };
    let file_a = Bitboard::all().file(File::A);
    let file_h = Bitboard::all().file(File::H);
    let rank_1 = Bitboard::all().rank(Rank::Rank1);
    let rank_8 = Bitboard::all().rank(Rank::Rank8);
    for sq_idx in (Square::A1 as u64)..=(Square::H8 as u64) {
        let sq = FromPrimitive::from_u64(sq_idx).unwrap();
        let mut board = Bitboard::none();
        if !rank_8.test(sq) {
            let north = FromPrimitive::from_u64(sq_idx + 8).unwrap();
            board.set(north);
            if !file_a.test(sq) {
                let nw = FromPrimitive::from_u64(sq_idx + 7).unwrap();
                board.set(nw);
            }

            if !file_h.test(sq) {
                let ne = FromPrimitive::from_u64(sq_idx + 9).unwrap();
                board.set(ne);
            }
        }

        if !rank_1.test(sq) {
            let south = FromPrimitive::from_u64(sq_idx - 8).unwrap();
            board.set(south );
            if !file_a.test(sq) {
                let sw = FromPrimitive::from_u64(sq_idx - 9).unwrap();
                board.set(sw);
            }

            if !file_h.test(sq) {
                let se = FromPrimitive::from_u64(sq_idx - 7).unwrap();
                board.set(se);
            }
        }

        if !file_a.test(sq) {
            let west = FromPrimitive::from_u64(sq_idx - 1).unwrap();
            board.set(west);
        }

        if !file_h.test(sq) {
            let east = FromPrimitive::from_u64(sq_idx + 1).unwrap();
            board.set(east);
        }

        king_table[sq_idx as usize] = board;
    }
}

/// Initializes all of the global precomputed state required for efficient
/// run-time lookups of sliding moves.
fn initialize_rays() {
    // the idea here is to generate rays in every direction for every square
    // on the board, to be used by the above methods.
    let ray_table = unsafe { &mut RAY_TABLE };
    for sq in (Square::A1 as usize)..=(Square::H8 as usize) {
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

/// Initialize all of the global lookup tables used for generating attack
/// moves.
pub fn initialize() {
    initialize_rays();
    initialize_knights();
    initialize_pawns();
    initialize_kings();
}

#[cfg(test)]
mod tests {
    use types::{Square, Direction, Color};
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

    #[test]
    fn center_knight() {
        super::initialize();
        let square = Square::E4;
        let moves = super::knight_attacks(square);
        println!("moves: ");
        println!("{}", moves);
        assert_eq!(8, moves.count());
        assert!(moves.test(Square::F6));
        assert!(moves.test(Square::G5));
        assert!(moves.test(Square::G3));
        assert!(moves.test(Square::F2));
        assert!(moves.test(Square::D6));
        assert!(moves.test(Square::C5));
        assert!(moves.test(Square::C3));
        assert!(moves.test(Square::D2));
    }

    #[test]
    fn edge_knight() {
        super::initialize();
        let square = Square::H8;
        let moves = super::knight_attacks(square);
        println!("moves: ");
        println!("{}", moves);
        assert_eq!(2, moves.count());
        assert!(moves.test(Square::G6));
        assert!(moves.test(Square::F7));
    }

    #[test]
    fn center_pawn_white() {
        super::initialize();
        let square = Square::E4;
        let moves = super::pawn_attacks(square, Color::White);
        println!("moves: ");
        println!("{}", moves);
        assert_eq!(2, moves.count());
        assert!(moves.test(Square::D5));
        assert!(moves.test(Square::F5));
    }

    #[test]
    fn center_pawn_black() {
        super::initialize();
        let square = Square::E4;
        let moves = super::pawn_attacks(square, Color::Black);
        println!("moves: ");
        println!("{}", moves);
        assert_eq!(2, moves.count());
        assert!(moves.test(Square::D3));
        assert!(moves.test(Square::F3));
    }

    #[test]
    fn edge_pawn() {
        super::initialize();
        let square = Square::A1;
        let moves = super::pawn_attacks(square, Color::White);
        assert_eq!(1, moves.count());
        assert!(moves.test(Square::B2));
    }

    #[test]
    fn center_king() {
        super::initialize();
        let moves = super::king_attacks(Square::E4);
        println!("moves: ");
        println!("{}", moves);
        assert_eq!(8, moves.count());
        assert!(moves.test(Square::E5));
        assert!(moves.test(Square::F5));
        assert!(moves.test(Square::D5));
        assert!(moves.test(Square::D4));
        assert!(moves.test(Square::D3));
        assert!(moves.test(Square::E3));
        assert!(moves.test(Square::F3));
    }

    #[test]
    fn ray_pin() {
        // this test exposes bugs in positive and negative ray piece blocker
        // generation
        super::initialize();
        let mut occ = Bitboard::none();
        occ.set(Square::E2);
        occ.set(Square::E1);
        let moves = super::rook_attacks(Square::E6, occ);
        println!("moves: ");
        println!("{}", moves);
        assert!(moves.test(Square::E5));
        assert!(moves.test(Square::E4));
        assert!(moves.test(Square::E3));
        assert!(moves.test(Square::E2));
        assert!(!moves.test(Square::E1));
    }

    #[bench]
    fn single_positive_ray_bench(b: &mut test::Bencher) {
        b.iter(|| {
                   let square = test::black_box(Square::E4);
                   let dir = test::black_box(Direction::NorthWest);
                   super::positive_ray_attacks(square, Bitboard::none(), dir);
               });
    }

    #[bench]
    fn single_negative_ray_bench(b: &mut test::Bencher) {
        b.iter(|| {
                   let square = test::black_box(Square::E4);
                   let dir = test::black_box(Direction::SouthWest);
                   super::negative_ray_attacks(square, Bitboard::none(), dir);
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

    #[bench]
    fn center_knight_bench(b: &mut Bencher) {
        b.iter(|| {
                   let square = test::black_box(Square::E4);
                   super::knight_attacks(square)
               })
    }
}

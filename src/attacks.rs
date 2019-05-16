// Copyright 2017-2019 Sean Gillespie.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.
use crate::bitboard::Bitboard;
use crate::bitboard::{
    BB_FILE_A, BB_FILE_AB, BB_FILE_GH, BB_FILE_H, BB_RANK_1, BB_RANK_12, BB_RANK_78, BB_RANK_8,
};
use crate::types::{Color, Direction, Square, TableIndex, COLORS, SQUARES};

struct KingTable {
    table: [Bitboard; 64],
}

impl KingTable {
    pub fn new() -> KingTable {
        let mut kt = KingTable {
            table: [Bitboard::none(); 64],
        };

        for &sq in SQUARES.iter() {
            let mut board = Bitboard::none();
            if !BB_RANK_8.test(sq) {
                board.set(sq.plus(8));
                if !BB_FILE_A.test(sq) {
                    board.set(sq.plus(7));
                }
                if !BB_FILE_H.test(sq) {
                    board.set(sq.plus(9));
                }
            }

            if !BB_RANK_1.test(sq) {
                board.set(sq.plus(-8));
                if !BB_FILE_A.test(sq) {
                    board.set(sq.plus(-9));
                }
                if !BB_FILE_H.test(sq) {
                    board.set(sq.plus(-7));
                }
            }

            if !BB_FILE_A.test(sq) {
                board.set(sq.plus(-1));
            }
            if !BB_FILE_H.test(sq) {
                board.set(sq.plus(1));
            }

            kt.table[sq.as_index() as usize] = board;
        }

        kt
    }

    pub fn attacks(&self, sq: Square) -> Bitboard {
        self.table[sq.as_index()]
    }
}

struct PawnTable {
    table: [[Bitboard; 2]; 64],
}

impl PawnTable {
    pub fn new() -> PawnTable {
        let mut pt = PawnTable {
            table: [[Bitboard::none(); 2]; 64],
        };

        for &sq in SQUARES.iter() {
            for &color in COLORS.iter() {
                let mut board = Bitboard::none();
                let (promo_rank, up_left, up_right) = match color {
                    Color::White => (BB_RANK_8, 7, 9),
                    Color::Black => (BB_RANK_1, -9, -7),
                };

                if promo_rank.test(sq) {
                    // No legal moves for this particular pawn. It's generally impossible
                    // for pawns to be on the promotion rank anyway since they should have
                    // been promoted already.
                    continue;
                }

                if !BB_FILE_A.test(sq) {
                    board.set(sq.plus(up_left));
                }
                if !BB_FILE_H.test(sq) {
                    board.set(sq.plus(up_right));
                }

                pt.table[sq.as_index()][color.as_index()] = board;
            }
        }

        pt
    }

    pub fn attacks(&self, sq: Square, color: Color) -> Bitboard {
        self.table[sq.as_index()][color.as_index()]
    }
}

struct KnightTable {
    table: [Bitboard; 64],
}

impl KnightTable {
    pub fn new() -> KnightTable {
        let mut kt = KnightTable {
            table: [Bitboard::none(); 64],
        };

        for &sq in SQUARES.iter() {
            let mut board = Bitboard::none();
            if !BB_FILE_A.test(sq) && !BB_RANK_78.test(sq) {
                board.set(sq.plus(15));
            }
            if !BB_FILE_H.test(sq) && !BB_RANK_78.test(sq) {
                board.set(sq.plus(17));
            }
            if !BB_FILE_GH.test(sq) && !BB_RANK_8.test(sq) {
                board.set(sq.plus(10));
            }
            if !BB_FILE_GH.test(sq) && !BB_RANK_1.test(sq) {
                board.set(sq.plus(-6));
            }
            if !BB_FILE_H.test(sq) && !BB_RANK_12.test(sq) {
                board.set(sq.plus(-15));
            }
            if !BB_FILE_A.test(sq) && !BB_RANK_12.test(sq) {
                board.set(sq.plus(-17));
            }
            if !BB_FILE_AB.test(sq) && !BB_RANK_1.test(sq) {
                board.set(sq.plus(-10));
            }
            if !BB_FILE_AB.test(sq) && !BB_RANK_8.test(sq) {
                board.set(sq.plus(6));
            }
            kt.table[sq.as_index()] = board;
        }
        kt
    }

    pub fn attacks(&self, sq: Square) -> Bitboard {
        self.table[sq.as_index()]
    }
}

struct RayTable {
    table: [[Bitboard; 8]; 65],
}

impl RayTable {
    pub fn new() -> RayTable {
        let mut rt = RayTable {
            table: [[Bitboard::none(); 8]; 65],
        };

        for &sq in SQUARES.iter() {
            let mut populate_dir = |dir: Direction, edge: Bitboard| {
                let mut entry = Bitboard::none();
                if edge.test(sq) {
                    // Nothing to do here, there are no legal moves on this ray from this square.
                    rt.table[sq.as_index()][dir.as_index()] = entry;
                    return;
                }

                // Starting at the given square, cast a ray in the given direction and add all bits to the ray mask.
                let mut cursor = sq;
                loop {
                    cursor = cursor.towards(dir);
                    entry.set(cursor);

                    // Did we reach the end of the board? If so, stop.
                    if edge.test(cursor) {
                        break;
                    }
                }
                rt.table[sq.as_index()][dir.as_index()] = entry;
            };

            populate_dir(Direction::North, BB_RANK_8);
            populate_dir(Direction::NorthEast, BB_RANK_8.or(BB_FILE_H));
            populate_dir(Direction::East, BB_FILE_H);
            populate_dir(Direction::SouthEast, BB_RANK_1.or(BB_FILE_H));
            populate_dir(Direction::South, BB_RANK_1);
            populate_dir(Direction::SouthWest, BB_RANK_1.or(BB_FILE_A));
            populate_dir(Direction::West, BB_FILE_A);
            populate_dir(Direction::NorthWest, BB_RANK_8.or(BB_FILE_A));
        }
        rt
    }

    pub fn attacks(&self, sq: usize, dir: Direction) -> Bitboard {
        self.table[sq.as_index()][dir.as_index()]
    }
}

lazy_static! {
    static ref KING_TABLE: KingTable = KingTable::new();
    static ref PAWN_TABLE: PawnTable = PawnTable::new();
    static ref KNIGHT_TABLE: KnightTable = KnightTable::new();
    static ref RAY_TABLE: RayTable = RayTable::new();
}

fn positive_ray_attacks(sq: Square, occupancy: Bitboard, dir: Direction) -> Bitboard {
    debug_assert!(dir.as_vector() > 0);
    let attacks = RAY_TABLE.attacks(sq.as_index(), dir);
    let blocker = attacks.and(occupancy).bits();
    let blocking_square = blocker.trailing_zeros() as usize;
    let blocking_ray = RAY_TABLE.attacks(blocking_square, dir);
    attacks.xor(blocking_ray)
}

fn negative_ray_attacks(sq: Square, occupancy: Bitboard, dir: Direction) -> Bitboard {
    debug_assert!(dir.as_vector() < 0);
    let attacks = RAY_TABLE.attacks(sq.as_index(), dir);
    let blocker = attacks.and(occupancy).bits();
    let blocking_square = (64 - blocker.leading_zeros()).checked_sub(1).unwrap_or(64) as usize;
    let blocking_ray = RAY_TABLE.attacks(blocking_square, dir);
    attacks.xor(blocking_ray)
}

fn diagonal_attacks(sq: Square, occupancy: Bitboard) -> Bitboard {
    positive_ray_attacks(sq, occupancy, Direction::NorthWest)
        | negative_ray_attacks(sq, occupancy, Direction::SouthEast)
}

fn antidiagonal_attacks(sq: Square, occupancy: Bitboard) -> Bitboard {
    positive_ray_attacks(sq, occupancy, Direction::NorthEast)
        | negative_ray_attacks(sq, occupancy, Direction::SouthWest)
}

fn file_attacks(sq: Square, occupancy: Bitboard) -> Bitboard {
    positive_ray_attacks(sq, occupancy, Direction::North)
        | negative_ray_attacks(sq, occupancy, Direction::South)
}

fn rank_attacks(sq: Square, occupancy: Bitboard) -> Bitboard {
    positive_ray_attacks(sq, occupancy, Direction::East)
        | negative_ray_attacks(sq, occupancy, Direction::West)
}

pub fn pawn_attacks(sq: Square, color: Color) -> Bitboard {
    PAWN_TABLE.attacks(sq, color)
}

pub fn bishop_attacks(sq: Square, occupancy: Bitboard) -> Bitboard {
    diagonal_attacks(sq, occupancy) | antidiagonal_attacks(sq, occupancy)
}

pub fn knight_attacks(sq: Square) -> Bitboard {
    KNIGHT_TABLE.attacks(sq)
}

pub fn rook_attacks(sq: Square, occupancy: Bitboard) -> Bitboard {
    file_attacks(sq, occupancy) | rank_attacks(sq, occupancy)
}

pub fn queen_attacks(sq: Square, occupancy: Bitboard) -> Bitboard {
    bishop_attacks(sq, occupancy) | rook_attacks(sq, occupancy)
}

pub fn king_attacks(sq: Square) -> Bitboard {
    KING_TABLE.attacks(sq)
}

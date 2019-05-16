// Copyright 2017-2019 Sean Gillespie.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.
use crate::bitboard::Bitboard;
use crate::bitboard::{
    BB_FILES, BB_FILE_A, BB_FILE_B, BB_FILE_C, BB_FILE_D, BB_FILE_E, BB_FILE_F, BB_FILE_G,
    BB_FILE_H, BB_RANKS,
};
use crate::move_generator::{MoveGenerator, MoveVec};
use crate::moves::Move;
use crate::position::Position;
use crate::types::{Color, File, FILES};

/// Provider of common board analyses upon a static position. It is suitable for use in board
/// evaluators, where analysis queries can be aggressively cached when evaluating a single,
/// immutable board position.
pub struct Analysis<'a> {
    pos: &'a Position,
}

impl<'a> Analysis<'a> {
    pub fn new(pos: &'a Position) -> Analysis<'a> {
        Analysis { pos }
    }

    /// Returns the set of doubled pawns left by the given color.
    pub fn doubled_pawns(&self, color: Color) -> Bitboard {
        let pawns = self.pos.pawns(color);
        let mut answer = Bitboard::none();
        for &file in &BB_FILES {
            let pawns_on_file = pawns.and(file);
            if pawns_on_file.count() > 1 {
                answer = answer.or(pawns_on_file);
            }
        }

        answer
    }

    /// Returns the set of backward pawns left by the given color.
    pub fn backward_pawns(&self, color: Color) -> Bitboard {
        fn walk_rank<I>(
            iter: I,
            current_file_pawns: Bitboard,
            adjacent_file_pawns: Bitboard,
        ) -> Bitboard
        where
            I: Iterator<Item = Bitboard>,
        {
            let mut answer = Bitboard::none();
            for rank in iter {
                let current_file_rank = rank.and(current_file_pawns);
                let adjacent_file_rank = rank.and(adjacent_file_pawns);
                if !current_file_rank.empty() && adjacent_file_rank.empty() {
                    answer = answer.or(current_file_rank);
                    break;
                }

                if !adjacent_file_rank.empty() && current_file_rank.empty() {
                    break;
                }
            }

            answer
        }

        let pawns = self.pos.pawns(color);
        let mut answer = Bitboard::none();
        for &file in &FILES {
            let adj_files = adjacent_files(file);
            let current_file = Bitboard::all().file(file);
            let pawns_on_current_file = pawns.and(current_file);
            let pawns_on_adjacent_files = pawns.and(adj_files);
            if pawns_on_current_file.empty() {
                continue;
            }

            let file_answer = match color {
                Color::White => walk_rank(
                    BB_RANKS.iter().cloned(),
                    pawns_on_current_file,
                    pawns_on_adjacent_files,
                ),
                Color::Black => walk_rank(
                    BB_RANKS.iter().cloned().rev(),
                    pawns_on_current_file,
                    pawns_on_adjacent_files,
                ),
            };

            answer = answer.or(file_answer);
        }

        answer
    }

    pub fn isolated_pawns(&self, color: Color) -> Bitboard {
        let pawns = self.pos.pawns(color);
        let mut answer = Bitboard::none();
        for &file in &FILES {
            let adj_files = adjacent_files(file);
            let current_file = Bitboard::all().file(file);
            let pawns_on_current_file = pawns.and(current_file);
            let pawns_on_adjacent_file = pawns.and(adj_files);
            if pawns_on_current_file.empty() {
                continue;
            }

            if pawns_on_adjacent_file.empty() {
                answer = answer.or(pawns_on_current_file);
            }
        }

        answer
    }

    pub fn mobility(&self, color: Color) -> u32 {
        // Our move generator only operates on the current side to move. If we need to analyze the
        // other side, make a null move and analyze that instead.
        let pos = if self.pos.side_to_move() != color {
            let mut copied_pos = self.pos.clone();
            copied_pos.apply_move(Move::null());
            copied_pos
        } else {
            self.pos.clone()
        };

        assert!(pos.side_to_move() == color);
        let mut move_vec = MoveVec::default();
        let gen = MoveGenerator::new();
        gen.generate_moves(&pos, &mut move_vec);
        let mut count = 0;
        for &mov in &move_vec {
            if pos.is_legal_given_pseudolegal(mov) {
                count += 1;
            }
        }

        count
    }
}

fn adjacent_files(file: File) -> Bitboard {
    match file {
        File::A => BB_FILE_B,
        File::B => BB_FILE_A.or(BB_FILE_C),
        File::C => BB_FILE_B.or(BB_FILE_D),
        File::D => BB_FILE_C.or(BB_FILE_E),
        File::E => BB_FILE_D.or(BB_FILE_F),
        File::F => BB_FILE_E.or(BB_FILE_G),
        File::G => BB_FILE_F.or(BB_FILE_H),
        File::H => BB_FILE_G,
    }
}

#[cfg(test)]
mod tests {
    use super::Analysis;

    use crate::position::Position;
    use crate::types::{Color, Square};

    #[test]
    fn doubled_pawn_smoke() {
        let pos = Position::from_fen("8/6P1/2P5/4P3/2P2P2/PP1P2P1/P7/8 w - - 0 1").unwrap();
        let analysis = Analysis::new(&pos);
        let doubled_pawns = analysis.doubled_pawns(Color::White);

        assert!(doubled_pawns.test(Square::A2));
        assert!(doubled_pawns.test(Square::A3));

        assert!(!doubled_pawns.test(Square::B3));

        assert!(doubled_pawns.test(Square::C4));
        assert!(doubled_pawns.test(Square::C6));

        assert!(!doubled_pawns.test(Square::D3));
        assert!(!doubled_pawns.test(Square::E5));
        assert!(!doubled_pawns.test(Square::F4));

        assert!(doubled_pawns.test(Square::G3));
        assert!(doubled_pawns.test(Square::G7));
    }

    #[test]
    fn backward_pawn_smoke() {
        let pos = Position::from_fen("8/8/8/8/8/2P1P3/3P4/8 w - - 0 1").unwrap();
        let analysis = Analysis::new(&pos);
        let backward_pawns = analysis.backward_pawns(Color::White);
        assert_eq!(1, backward_pawns.count());
        assert!(backward_pawns.test(Square::D2));
    }

    #[test]
    fn backward_pawn_smoke_black() {
        let pos = Position::from_fen("8/3p4/2p1p3/8/8/8/8/8 b - - 0 1").unwrap();
        let analysis = Analysis::new(&pos);
        let backward_pawns = analysis.backward_pawns(Color::Black);
        assert_eq!(1, backward_pawns.count());
        assert!(backward_pawns.test(Square::D7));
    }

    #[test]
    fn mobility_smoke() {
        let pos = Position::from_fen("8/8/4r3/8/8/4B3/4K3/8 w - - 0 1").unwrap();
        let analysis = Analysis::new(&pos);

        // White's bishop is not allowed to move at all, since it is absolutely pinned by the Black
        // rook. As a result, its mobility score is low, despite having more pieces on the board.
        assert_eq!(7, analysis.mobility(Color::White));
        assert_eq!(12, analysis.mobility(Color::Black));
    }

    #[test]
    fn isolated_pawn_smoke() {
        let pos = Position::from_fen("8/8/8/8/8/3P1P2/6P1/8 w - - 0 1").unwrap();
        let analysis = Analysis::new(&pos);
        let isolated_pawns = analysis.isolated_pawns(Color::White);
        assert_eq!(1, isolated_pawns.count());
        assert!(isolated_pawns.test(Square::D3));
    }
}

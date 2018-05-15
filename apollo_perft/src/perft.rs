// Copyright 2017 Sean Gillespie.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.
use apollo::Position;

#[derive(Serialize)]
pub struct IntermediatePosition {
    fen: String,
    moves: Vec<String>,
}

pub fn perft(pos: Position, depth: u32, save_ints: bool) -> (u64, Vec<IntermediatePosition>) {
    let mut positions = vec![];
    let results = perft_impl(pos, depth, &mut positions, save_ints);
    (results, positions)
}

fn perft_impl(pos: Position,
              depth: u32,
              positions: &mut Vec<IntermediatePosition>,
              save_ints: bool)
              -> u64 {
    if depth == 0 {
        return 1;
    }

    let mut nodes = 0;
    let fen = pos.as_fen();
    let mut int = IntermediatePosition {
        fen: fen,
        moves: vec![],
    };
    for mov in pos.pseudolegal_moves() {
        let mut new_pos = pos.clone();
        let to_move = new_pos.side_to_move();
        new_pos.apply_move(mov);
        if !new_pos.is_check(to_move) {
            if save_ints {
                int.moves.push(mov.as_uci());
            }
            nodes += perft_impl(new_pos, depth - 1, positions, save_ints);
        }
    }

    if save_ints {
        positions.push(int);
    }
    nodes
}

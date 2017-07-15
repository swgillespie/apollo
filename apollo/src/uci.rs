// Copyright 2017 Sean Gillespie.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.
use apollo_engine::Move;
use logger;
use engine;

pub fn handshake() {
    println!("id name {} {}",
             env!("CARGO_PKG_NAME"),
             env!("CARGO_PKG_VERSION"));
    println!("id author {}", env!("CARGO_PKG_AUTHORS"));
    println!("uciok");
}

pub fn debug(tok: &[&str]) {
    assert!(tok[0] == "debug");
    if tok.len() < 2 {
        // not valid, just roll with it.
        return;
    }

    if tok[1] == "on" {
        logger::debug_enable();
        info!("verbose debug logging enabled");
    } else if tok[1] == "off" {
        info!("verbose debug logging disabled");
        logger::debug_disable();
    }
}

pub fn isready() {
    println!("isready");
}

pub fn setoption(_tokens: &[&str]) {
    // we don't have any options yet...
}

pub fn newgame() {
    engine::new_game();
}

pub fn position(tokens: &[&str]) {
    assert_eq!("position", tokens[0]);
    let (moves_start, fen) = {
        let mut moves_start = None;
        let mut seen_startpos = false;
        let mut fen_components = vec![];
        for (i, token) in tokens[1..].iter().enumerate() {
            if *token == "moves" {
                moves_start = Some(i);
                break;
            }

            if *token == "startpos" {
                seen_startpos = true;
            }

            fen_components.push(*token);
        }

        let fen = if seen_startpos {
            "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1".to_string()
        } else {
            fen_components.join(" ")
        };

        (moves_start, fen)
    };

    let moves = {
        if let Some(moves_index) = moves_start {
            let mut list = vec![];
            for token in &tokens[moves_index+1..] {
                list.push(*token);
            }

            list
        } else {
            vec![]
        }
    };

    engine::new_position(&fen, &moves);
}

pub fn bestmove(mov: Move) {
    println!("bestmove {}", mov.as_uci());
}

pub fn go(_tokens: &[&str]) {
    engine::go();
}
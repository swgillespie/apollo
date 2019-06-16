// Copyright 2017-2019 Sean Gillespie.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.
use std::fmt::Write;
use std::io::{self, BufRead};
use std::time::Duration;

use crate::eval::Score;
use crate::eval::ShannonEvaluator;
use crate::position::Position;
use crate::search::{NullDataRecorder, Searcher};

macro_rules! uci_println {
    ($fmt:expr) => {
        {
            info!(concat!("=> ", $fmt));
            println!($fmt);
        }
    };
    ($fmt:expr, $($arg:tt)*) => {
        {
            info!(concat!("=> ", $fmt), $($arg)*);
            println!($fmt, $($arg)*);
        }
    };
}

pub struct UciServer {
    pos: Position,
    search: Searcher<ShannonEvaluator>,
}

impl UciServer {
    pub fn new() -> UciServer {
        UciServer {
            pos: Position::new(),
            search: Default::default(),
        }
    }

    pub fn run(mut self) -> io::Result<()> {
        let locked_stdin = io::stdin();
        let stdin = locked_stdin.lock();
        for maybe_line in stdin.lines() {
            let line = maybe_line?;
            info!("<= {}", line);
            let components: Vec<_> = line.split_whitespace().collect();
            let (&first, rest) = components.split_first().unwrap_or((&"", &[]));
            match (first, rest) {
                ("uci", []) => self.handle_uci(),
                ("isready", []) => uci_println!("readyok"),
                ("debug", ["on"]) => {}
                ("debug", ["off"]) => {}
                ("quit", []) => return Ok(()),
                ("ucinewgame", []) => {
                    info!("clearing search tables");
                    self.search = Default::default();
                }
                ("position", args) => self.handle_position(args),
                ("go", args) => self.handle_go(args),
                ("stop", []) => {}
                _ => uci_println!("unrecognized command"),
            }
        }

        Ok(())
    }

    fn handle_uci(&self) {
        uci_println!(
            "id name {} {}",
            env!("CARGO_PKG_NAME"),
            env!("CARGO_PKG_VERSION")
        );
        uci_println!("id author {}", env!("CARGO_PKG_AUTHORS"));
        uci_println!("uciok");
    }

    fn handle_position(&mut self, slice: &[&str]) {
        let move_idx = slice
            .into_iter()
            .position(|&idx| idx == "moves")
            .unwrap_or(slice.len() - 1);
        let moves = &slice[move_idx + 1..];

        let fen_idx = slice.into_iter().position(|&idx| idx == "fen");
        let startpos_idx = slice.into_iter().position(|&idx| idx == "startpos");
        let fen = if let Some(idx) = fen_idx {
            let s = &slice[idx + 1..move_idx];
            s.join(" ")
        } else if let Some(_) = startpos_idx {
            "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1".to_owned()
        } else {
            uci_println!("invalid position command");
            return;
        };

        self.pos = if let Ok(pos) = Position::from_fen(&fen) {
            pos
        } else {
            uci_println!("invalid fen");
            return;
        };

        debug!("moves: {:?}", moves);
        debug!("fen: {}", fen);
        for mov in moves {
            if let Some(parsed_move) = self.pos.move_from_uci(mov) {
                assert!(self.pos.is_legal(parsed_move));
                self.pos.apply_move(parsed_move);
            } else {
                info!("invalid move: {}", mov);
            }
        }
    }

    fn handle_go(&mut self, _: &[&str]) {
        info!("beginning search, (depth 10, 20 second budget)");
        let result = self.search.search(
            &self.pos,
            10,
            Some(Duration::from_secs(1)),
            &NullDataRecorder,
        );
        info!("move: {} ({})", result.best_move, result.score);

        let mut out = String::new();
        write!(&mut out, "info depth 5 nodes {}", result.nodes_searched).unwrap();
        match result.score {
            Score::Evaluated(score) => write!(&mut out, " score cp {}", score).unwrap(),
            Score::Win(moves) => write!(&mut out, " score mate {}", moves).unwrap(),
            Score::Loss(moves) => write!(&mut out, " score mate -{}", moves).unwrap(),
        }
        uci_println!("{}", out);
        uci_println!("bestmove {}", result.best_move);
    }
}

impl Default for UciServer {
    fn default() -> UciServer {
        UciServer::new()
    }
}

/*
pub struct UciServer {
    pos: Position,
    searcher: IterativeDeepeningSearcher<ShannonEvaluator>,
}

impl UciServer {
    pub fn new() -> UciServer {
        UciServer {
            pos: Position::new(),
            searcher: IterativeDeepeningSearcher::new(),
        }
    }

    pub fn run<R, W, L>(mut self, reader: R, mut writer: W, mut log: L) -> io::Result<()>
    where
        R: BufRead,
        W: Write,
        L: Write,
    {
        for maybe_line in reader.lines() {
            let line = maybe_line?;
            writeln!(&mut log, "{}", line)?;
            let components: Vec<_> = line.split_whitespace().collect();
            match components.split_first().unwrap_or((&"", &[])) {
                (&"uci", []) => self.handle_uci(&mut writer)?,
                (&"isready", []) => writeln!(&mut writer, "readyok")?,
                (&"debug", ["on"]) => {}
                (&"debug", ["off"]) => {}
                (&"quit", []) => break,
                (&"ucinewgame", []) => {}
                (&"go", args) => self.handle_go(&mut writer, &mut log, args)?,
                (&"position", args) => self.handle_position(&mut writer, &mut log, args)?,
                _ => writeln!(&mut writer, "unrecognized command")?,
            }
        }

        Ok(())
    }

    fn handle_uci<W: Write>(&mut self, w: &mut W) -> io::Result<()> {
        writeln!(
            w,
            "id name {} {}",
            env!("CARGO_PKG_NAME"),
            env!("CARGO_PKG_VERSION")
        )?;
        writeln!(w, "id author {}", env!("CARGO_PKG_AUTHORS"),)?;
        writeln!(w, "uciok")
    }

    fn handle_position<W: Write, L: Write>(
        &mut self,
        w: &mut W,
        log: &mut L,
        slice: &[&str],
    ) -> io::Result<()> {
        let move_idx = slice
            .into_iter()
            .position(|&idx| idx == "moves")
            .unwrap_or(slice.len() - 1);
        let moves = &slice[move_idx + 1..];

        let fen_idx = slice.into_iter().position(|&idx| idx == "fen");
        let startpos_idx = slice.into_iter().position(|&idx| idx == "startpos");
        let fen = if let Some(idx) = fen_idx {
            let s = &slice[idx + 1..move_idx];
            s.join(" ")
        } else if let Some(_) = startpos_idx {
            "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1".to_owned()
        } else {
            writeln!(w, "invalid position command")?;
            return Ok(());
        };

        self.pos = if let Ok(pos) = Position::from_fen(&fen) {
            pos
        } else {
            writeln!(w, "invalid fen")?;
            return Ok(());
        };

        //writeln!(log, "moves: {:?}", moves)?;
        //writeln!(log, "fen: {}", fen)?;
        for mov in moves {
            if let Some(parsed_move) = self.pos.move_from_uci(mov) {
                // writeln!(log, "applying move: {}", mov)?;
                assert!(self.pos.is_legal(parsed_move));
                self.pos.apply_move(parsed_move);
            } else {
                writeln!(log, "invalid move: {}", mov)?;
            }
        }

        writeln!(log, "{}", self.pos)?;
        Ok(())
    }

    fn handle_go<W: Write, L: Write>(
        &mut self,
        w: &mut W,
        l: &mut L,
        _: &[&str],
    ) -> io::Result<()> {
        writeln!(l, "beginning search")?;
        let result = self
            .searcher
            .search(&self.pos, 10, Some(Duration::from_secs(20)));
        writeln!(l, "move: {} ({})", result.best_move, result.score)?;
        write!(w, "info depth 5 nodes {}", result.nodes_searched)?;
        match result.score {
            Score::Evaluated(score) => write!(w, " score cp {}", score)?,
            Score::Win(moves) => write!(w, " score mate {}", moves)?,
            Score::Loss(moves) => write!(w, " score mate -{}", moves)?,
        }
        writeln!(w)?;
        writeln!(w, "bestmove {}", result.best_move)?;
        Ok(())
    }
}

*/

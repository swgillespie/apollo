// Copyright 2017-2019 Sean Gillespie.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

#[macro_use]
extern crate clap;

use std::fs::File;
use std::process;
use std::time::Instant;

use apollo::book::OpeningBook;
use apollo::eval::ShannonEvaluator;
use apollo::search::{CsvDataRecorder, Searcher};
use apollo::uci::UciServer;
use apollo::{perft, Position};
use clap::{App, Arg, ArgMatches, SubCommand};

fn main() {
    env_logger::init();
    let matches = App::new(crate_name!())
        .version(crate_version!())
        .author(crate_authors!())
        .about(crate_description!())
        .subcommand(
            SubCommand::with_name("perft")
                .about("PERFT analysis of board positions")
                .arg(
                    Arg::with_name("FEN")
                        .help("FEN string for a board position")
                        .required(true)
                        .index(1),
                )
                .arg(
                    Arg::with_name("depth")
                        .help("Depth of move tree to search")
                        .value_name("DEPTH")
                        .short("-d")
                        .long("--depth")
                        .takes_value(true),
                ),
        )
        .subcommand(
            SubCommand::with_name("evaluate")
                .about("Evaluate a board position")
                .arg(
                    Arg::with_name("FEN")
                        .help("FEN string for a board position")
                        .required(true)
                        .index(1),
                )
                .arg(
                    Arg::with_name("depth")
                        .help("Depth of move tree to search")
                        .value_name("DEPTH")
                        .short("-d")
                        .long("--depth")
                        .takes_value(true),
                ),
        )
        .get_matches();

    if let Some(matches) = matches.subcommand_matches("perft") {
        run_perft(matches);
    }

    if let Some(matches) = matches.subcommand_matches("evaluate") {
        run_evaluate(matches);
    }

    let book = if let Ok(mut file) = File::open("book.json") {
        serde_json::from_reader::<_, OpeningBook>(&mut file).ok()
    } else {
        None
    };
    let svr = UciServer::new(book);
    svr.run().unwrap()
}

fn run_perft(matches: &ArgMatches) -> ! {
    let fen = matches.value_of("FEN").unwrap();
    let depth = value_t_or_exit!(matches, "depth", u32);
    let pos = match Position::from_fen(fen) {
        Ok(pos) => pos,
        Err(_) => {
            println!("invalid fen!");
            process::exit(1);
        }
    };

    println!("fen:   {}", fen);
    println!("depth: {}", depth);
    println!();
    println!("{}", pos);
    println!();
    for i in 1..depth + 1 {
        let start = Instant::now();
        let results = perft(&pos, i, true);
        let stop = Instant::now();
        let duration = stop - start;
        let nanos = duration.subsec_nanos() as u64;
        let ms = (1000 * 1000 * 1000 * duration.as_secs() + nanos) / (1000 * 1000);
        println!("perft({}) = {} ({} ms)", i, results, ms);
    }

    process::exit(0);
}

fn run_evaluate(matches: &ArgMatches) -> ! {
    let fen = matches.value_of("FEN").unwrap();
    let depth = value_t_or_exit!(matches, "depth", u32);
    let pos = match Position::from_fen(fen) {
        Ok(pos) => pos,
        Err(_) => {
            println!("invalid fen!");
            process::exit(1);
        }
    };

    println!("fen:   {}", fen);
    println!("depth: {}", depth);
    println!();
    println!("{}", pos);
    println!();

    let recorder = CsvDataRecorder::new(File::create("data.csv").unwrap());
    let mut searcher: Searcher<ShannonEvaluator> = Searcher::new(None);
    let result = searcher.search(&pos, depth, None, &recorder);
    println!("best move: {}", result.best_move);
    println!("    score: {}", result.score);
    println!("    nodes: {}", result.nodes_searched);
    process::exit(0);
}

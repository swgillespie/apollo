// Copyright 2017 Sean Gillespie.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.
extern crate apollo;
#[macro_use]
extern crate clap;
#[macro_use]
extern crate serde_derive;
extern crate serde;
extern crate serde_json;

mod perft;

use apollo::Position;
use std::time::Instant;
use std::process;
use std::fs::File;
use clap::{Arg, App};

fn main() {
    apollo::initialize();
    let matches = App::new(env!("CARGO_PKG_NAME"))
        .version(crate_version!())
        .author(crate_authors!())
        .about("PERFT calculator for chess positions")
        .arg(Arg::with_name("FEN")
                 .help("FEN representation of the position to calculate")
                 .required(true)
                 .index(1))
        .arg(Arg::with_name("depth")
                 .help("Depth of move tree to search")
                 .value_name("DEPTH")
                 .short("-d")
                 .long("--depth")
                 .takes_value(true))
        .arg(Arg::with_name("save")
                .help("Saves all intermediate positions as JSON (for move generator debugging)")
                .short("-s")
                .value_name("FILE")
                .long("--save-intermediates")
                .takes_value(true))
        .get_matches();
    let fen = matches.value_of("FEN").unwrap();
    let depth = value_t_or_exit!(matches, "depth", u32);
    let save_ints = matches.value_of("save");
    println!("fen:   {}", fen);
    println!("depth: {}", depth);
    if depth > 6 {
        println!("warning, this is probably going to take a while...");
    }

    let pos = match Position::from_fen(fen) {
        Ok(pos) => pos,
        Err(_) => {
            println!("invalid FEN!");
            process::exit(1);
        }
    };

    if let Some(ints_file) = save_ints {
        let (results, ints) = perft::perft(pos, depth, true);
        println!("perft({}) = {}", depth, results);
        println!("saving intermediate move states to disk...");
        let mut file = File::create(ints_file).unwrap();
        serde_json::to_writer_pretty(&mut file, &ints).unwrap();
        process::exit(0);
    }

    for i in 1..depth + 1 {
        let start = Instant::now();
        let (results, _) = perft::perft(pos.clone(), i, false);
        let stop = Instant::now();
        let duration = stop - start;
        let nanos = duration.subsec_nanos() as u64;
        let ms = (1000 * 1000 * 1000 * duration.as_secs() + nanos) / (1000 * 1000);
        println!("perft({}) = {} ({} ms)", i, results, ms);
    }
    process::exit(0);
}

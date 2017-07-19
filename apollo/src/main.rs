// Copyright 2017 Sean Gillespie.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.
#![feature(const_fn)]

extern crate apollo_engine;
extern crate parking_lot;
extern crate rand;
#[macro_use]
extern crate log;
#[macro_use]
extern crate lazy_static;
extern crate cancellation;

use std::io::{self, BufRead, Write};
use std::error::Error;
use std::process;

mod logger;
mod uci;
mod engine;
mod search;

fn main() {
    logger::initialize();
    if cfg!(debug_assertions) {
        logger::debug_enable();
    }

    engine::initialize();
    println!("Apollo chess engine, by Sean Gillespie");
    main_loop();
}

fn quit() -> ! {
    engine::shutdown();
    process::exit(0);
}

fn error_and_exit<E: Error>(err: E) -> ! {
    println!("info string fatal i/o error: {}", err.description());
    io::stdout().flush().unwrap();
    engine::shutdown();
    process::exit(1);
}

fn main_loop() {
    let stdin_ref = io::stdin();
    let mut stdin = stdin_ref.lock();

    uci::newgame();
    loop {
        let mut buf = String::new();
        match stdin.read_line(&mut buf) {
            Ok(0) => quit(),
            Ok(_) => {},
            Err(e) => error_and_exit(e)
        }

        let tokens : Vec<_> = buf.split_whitespace().collect();
        if tokens.len() == 0 {
            continue;
        }

        match tokens[0].as_ref() {
            "uci" => uci::handshake(),
            "debug" => uci::debug(&tokens),
            "quit" => quit(),
            "readyok" => uci::isready(),
            "setoption" => uci::setoption(&tokens),
            "ucinewgame" => uci::newgame(),
            "position" => uci::position(&tokens),
            "go" => uci::go(&tokens),
            "stop" => uci::stop(),
            // nonstandard uci commands, for debugging
            "currentpos" => uci::currentpos(),
            _ => println!("unknown command: {}", tokens[0])
        }
    }
}

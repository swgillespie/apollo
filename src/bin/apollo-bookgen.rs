// Copyright 2017-2019 Sean Gillespie.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

#[macro_use]
extern crate clap;

#[macro_use]
extern crate pest_derive;

use std::fs::File;
use std::io::Read;

use clap::{App, Arg};
use pest::Parser;

use apollo::book::{BookEntry, OpeningBook};
use apollo::{Move, Position, Square};

#[derive(Parser)]
#[grammar = "bin/pgn.pest"]
pub struct PGNParser;

fn main() {
    env_logger::init();
    let matches = App::new("apollo-bookgen")
        .version(crate_version!())
        .author(crate_authors!())
        .about("Generates an opening book for apollo from PGN databases")
        .arg(
            Arg::with_name("FILE")
                .help("PGN file to load games from")
                .required(true)
                .index(1),
        )
        .get_matches();

    let mut file = File::open(matches.value_of("FILE").unwrap()).unwrap();
    let mut buf = Vec::new();
    file.read_to_end(&mut buf).unwrap();
    let contents = String::from_utf8_lossy(&buf);

    let mut lines = match PGNParser::parse(Rule::lines, &contents) {
        Ok(result) => result,
        Err(e) => panic!("{}", e),
    };

    let mut book = OpeningBook::new();
    for line in lines.next().unwrap().into_inner() {
        let mut pos = Position::from_start_position();
        let mut book_entry = BookEntry {
            category: String::new(),
            lead_name: String::new(),
            response_name: None,
        };
        let mut move_sequence = vec![];
        for thing in line.into_inner() {
            match thing.as_rule() {
                Rule::tag_pair => {
                    let mut tag_name = None;
                    for tag in thing.into_inner() {
                        match tag.as_rule() {
                            Rule::identifier => {
                                tag_name = Some(tag.as_str().to_owned());
                            }
                            Rule::string => {
                                let tag_value = tag.as_str();
                                let trimmed = &tag_value[1..tag_value.len() - 1];
                                match &**tag_name.as_ref().unwrap() {
                                    "Site" => book_entry.category = trimmed.to_owned(),
                                    "White" => book_entry.lead_name = trimmed.to_owned(),
                                    "Black" => book_entry.response_name = Some(trimmed.to_owned()),
                                    _ => {}
                                }
                            }
                            _ => unreachable!(),
                        }
                    }
                }
                Rule::moves => {
                    for mov in thing.into_inner() {
                        for turn in mov.into_inner() {
                            match turn.as_rule() {
                                Rule::normal_move | Rule::castle => {
                                    let apollo_mov = pos.move_from_san(turn.as_str()).unwrap();
                                    move_sequence.push(apollo_mov);
                                    pos.apply_move(apollo_mov);
                                }
                                _ => {}
                            }
                        }
                    }
                }
                _ => unreachable!(),
            }
        }

        book.add_entry(&move_sequence, book_entry);
    }

    /*
    println!(
        "{:#?}",
        book.book_moves(&[Move::quiet(Square::E2, Square::E4)])
    );
    */
    println!("{}", serde_json::to_string_pretty(&book).unwrap());
}

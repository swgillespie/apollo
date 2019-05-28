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

use clap::{App, Arg};
use csv::Reader;

use apollo::search::Record;
use apollo::Position;

struct AnalysisRecord {
    fen: String,
    records: Vec<Record>,
}

fn main() {
    env_logger::init();
    let matches = App::new("apollo-data")
        .version(crate_version!())
        .author(crate_authors!())
        .about("Data analysis tool for apollo")
        .arg(
            Arg::with_name("FILE")
                .help("CSV file to load for analysis")
                .required(true)
                .index(1),
        )
        .get_matches();

    let file = File::open(matches.value_of("FILE").unwrap()).unwrap();
    let records = read_file(&file);
    let position = Position::from_fen(&records.fen).unwrap();
    println!("{}", position);
    println!("fen: {}", records.fen);
    print_records(&records);
}

fn read_file(file: &File) -> AnalysisRecord {
    let mut records = vec![];
    let mut reader = Reader::from_reader(file);
    for result in reader.deserialize() {
        let record: Record = result.unwrap();
        records.push(record);
    }

    AnalysisRecord {
        fen: records[0].fen.clone(),
        records,
    }
}

fn print_records(rec: &AnalysisRecord) {
    let mut prev_nodes: u64 = 0;
    for record in &rec.records {
        println!("-------------------------");
        println!("depth: {}", record.depth);
        println!("nodes: {}", record.nodes);
        if record.depth > 1 {
            println!(
                "effective branching factor: {}",
                record.nodes as f64 / prev_nodes as f64
            );
        }

        println!();
        println!(" pv nodes: {}", record.pv_nodes);
        println!("all nodes: {}", record.all_nodes);
        println!("cut nodes: {}", record.cut_nodes);
        println!();
        println!("tt absolute hits: {}", record.tt_absolute_hit);
        println!("      hash moves: {}", record.hash_move_node);
        println!("hash move cutoff: {}", record.hash_move_beta_cutoff);
        println!(" hash move alpha: {}", record.hash_move_improved_alpha);
        prev_nodes = record.nodes;
    }
}

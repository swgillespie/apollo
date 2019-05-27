// Copyright 2017-2019 Sean Gillespie.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.
use std::io::Write;
use std::sync::Mutex;

use csv::Writer;

use crate::position::Position;

#[derive(Clone, Default, Serialize, Deserialize)]
pub struct Record {
    pub fen: String,
    pub depth: u32,
    pub nodes: u64,
    pub pv_nodes: u64,
    pub all_nodes: u64,
    pub cut_nodes: u64,

    pub tt_absolute_hit_pv: u64,
    pub tt_absolute_hit_cut: u64,
    pub tt_absolute_hit_cut_improved_alpha: u64,
    pub tt_absolute_hit_all: u64,
    pub tt_absolute_hit: u64,

    pub hash_move_node: u64,
    pub hash_move_beta_cutoff: u64,
    pub hash_move_improved_alpha: u64,
}

pub trait DataRecorder {
    fn record(&self, pos: &Position, rec: &Record);
}

pub struct NullDataRecorder;
impl DataRecorder for NullDataRecorder {
    fn record(&self, _pos: &Position, _rec: &Record) {}
}

pub struct CsvDataRecorder<W: Write> {
    writer: Mutex<Writer<W>>,
}

impl<W: Write> CsvDataRecorder<W> {
    pub fn new(writer: W) -> CsvDataRecorder<W> {
        CsvDataRecorder {
            writer: Mutex::new(Writer::from_writer(writer)),
        }
    }
}

impl<W: Write> DataRecorder for CsvDataRecorder<W> {
    fn record(&self, pos: &Position, rec: &Record) {
        let mut copy = rec.clone();
        copy.fen = pos.as_fen();
        let mut writer = self.writer.lock().unwrap();
        writer.serialize(&copy).unwrap();
        writer.flush().unwrap();
    }
}

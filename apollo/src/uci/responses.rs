// Copyright 2017-2018 Sean Gillespie.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.
use std::io::{self, Write};

pub enum UciResponse {
    IdName(String),
    IdAuthor(String),
    Uciok,
    Readyok
}

impl UciResponse {
    pub fn render<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        match *self {
            UciResponse::IdName(ref name) => writeln!(writer, "id name {}", name),
            UciResponse::IdAuthor(ref author) => writeln!(writer, "id author {}", author),
            UciResponse::Uciok => writeln!(writer, "uciok"),
            UciResponse::Readyok => writeln!(writer, "readyok"),
            _ => unimplemented!()
        }
    }
}

// Copyright 2017-2018 Sean Gillespie.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.
use attacks::AttackTable;
use position::{Position, FenParseError};
use zobrist::ZobristHasher;

pub struct Engine {
    attack_table: Box<AttackTable>,
    hasher: Box<ZobristHasher>
}

impl Engine {
    pub fn new() -> Engine {
        Engine {
            attack_table: Box::new(AttackTable::new()),
            hasher: Box::new(ZobristHasher::new())
        }
    }

    pub(crate) fn attack_table(&self) -> &AttackTable {
        &self.attack_table
    }

    pub(crate) fn hasher(&self) -> &ZobristHasher {
        &self.hasher
    }

    pub fn new_position<'e>(&'e self) -> Position<'e> {
        Position::new(self)
    }

    pub fn new_position_from_fen<'e, S: AsRef<str>>(&'e self, fen: S) -> Result<Position<'e>, FenParseError> {
        Position::from_fen(self, fen)
    }
}

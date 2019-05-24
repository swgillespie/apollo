// Copyright 2017-2019 Sean Gillespie.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.
use hashbrown::HashMap;
use std::sync::atomic::AtomicU64;
use std::sync::RwLock;

use crate::eval::Score;
use crate::moves::Move;
use crate::position::Position;

#[derive(Copy, Clone, Debug)]
pub enum NodeKind {
    PrincipalVariation(Score),
    All(Score),
    Cut(Score),
}

#[derive(Clone, Debug)]
pub struct TableEntry {
    pub zobrist_key: u64,
    pub best_move: Option<Move>,
    pub depth: u32,
    pub node: NodeKind,
}

pub struct TableStats {
    table_hits: AtomicU64,
    table_misses: AtomicU64,
}

pub struct TranspositionTable {
    table: RwLock<HashMap<u64, TableEntry>>,
    stats: TableStats,
}

impl TranspositionTable {
    pub fn new() -> TranspositionTable {
        TranspositionTable {
            table: RwLock::new(HashMap::new()),
            stats: TableStats {
                table_hits: AtomicU64::new(0),
                table_misses: AtomicU64::new(0),
            },
        }
    }

    pub fn stats(&self) -> &TableStats {
        &self.stats
    }

    pub fn query<F, R>(&self, pos: &Position, f: F) -> R
    where
        F: FnOnce(Option<&TableEntry>) -> R,
    {
        let key = pos.zobrist_hash();
        let table = self.table.read().expect("T-Table lock was poisoned");
        let entry = table.get(&key);
        f(entry)
    }

    pub fn query_copy(&self, pos: &Position) -> Option<TableEntry> {
        self.query(pos, |entry| entry.cloned())
    }

    pub fn record_principal_variation(
        &self,
        pos: &Position,
        best_move: Move,
        depth: u32,
        score: Score,
    ) {
        let key = pos.zobrist_hash();
        let entry = TableEntry {
            zobrist_key: key,
            best_move: Some(best_move),
            depth: depth,
            node: NodeKind::PrincipalVariation(score),
        };
        self.record_entry(entry);
    }

    pub fn record_cut(&self, pos: &Position, best_move: Move, depth: u32, score: Score) {
        let key = pos.zobrist_hash();
        let entry = TableEntry {
            zobrist_key: key,
            best_move: Some(best_move),
            depth: depth,
            node: NodeKind::Cut(score),
        };
        self.record_entry(entry);
    }

    pub fn record_all(&self, pos: &Position, depth: u32, score: Score) {
        let key = pos.zobrist_hash();
        let entry = TableEntry {
            zobrist_key: key,
            best_move: None,
            depth: depth,
            node: NodeKind::All(score),
        };
        self.record_entry(entry);
    }

    fn record_entry(&self, entry: TableEntry) {
        let mut table = self.table.write().expect("T-Table lock was poisoned");
        table.insert(entry.zobrist_key, entry);
    }
}

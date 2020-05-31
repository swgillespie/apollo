// Copyright 2017-2019 Sean Gillespie.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.
#![allow(dead_code)]

#[macro_use]
extern crate num_derive;
#[macro_use]
extern crate bitflags;
#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate log;
#[macro_use]
extern crate serde_derive;

mod analysis;
pub mod attacks;
mod bitboard;
mod book;
pub mod eval;
mod move_generator;
mod moves;
mod perft;
mod position;
pub mod search;
mod types;
pub mod uci;
mod zobrist;

pub use bitboard::{Bitboard, BitboardIterator};
pub use move_generator::{MoveGenerator, MoveVec};
pub use moves::Move;
pub use perft::perft;
pub use position::Position;
pub use types::{Color, File, PieceKind, Rank, Square};

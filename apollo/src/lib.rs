// Copyright 2017 Sean Gillespie.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! ## The Apollo Chess Engine
//!
//! This crate provides an implementation of the rules of chess, complete
//! enough to form the foundation of a fully-fledged chess engine. It provides
//! the following services:
//!
//!   * Representation of chess positions, including parsing FEN notation
//!   * Pseudo-legal move generation for chess positions
//!   * Application of moves to chess positions
//!   * Check detection
//!
//! It is one component of the Apollo chess engine. Other components are
//! responsible for searching for good moves and communicating with a chess UI.
#![allow(unused_features)]
#![feature(const_fn, test)]

#[macro_use]
extern crate bitflags;
extern crate num_traits;
extern crate rand;

#[cfg(test)]
extern crate test;

mod engine;
mod types;
mod bitboard;
mod attacks;
mod moves;
mod position;
mod movegen;
mod zobrist;

pub use types::{Square, Rank, File, Color, Piece, PieceKind};
pub use moves::Move;
pub use position::{Position, FenParseError};
pub use bitboard::{Bitboard, BitboardIterator};
pub use engine::Engine;

pub fn initialize() {
    attacks::initialize();
    zobrist::initialize();
}

// Copyright 2017 Sean Gillespie. See the COPYRIGHT
// file at the top-level directory of this distribution.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.
#![allow(unused_features)]
#![feature(const_fn, inclusive_range_syntax, test)]

#[macro_use]
extern crate bitflags;
extern crate num_traits;
extern crate parking_lot;

#[cfg(test)]
extern crate test;

pub mod types;
pub mod bitboard;
pub mod slides;
pub mod moves;
pub mod position;
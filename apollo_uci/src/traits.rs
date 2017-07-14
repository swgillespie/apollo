// Copyright 2017 Sean Gillespie.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.
use std::io::{Write, Error};

// not sure if this trait is actually useful yet...
pub trait ToUciWire {
    fn to_uci_wire<W: Write>(&self, writer: &mut W) -> Result<(), Error>;
}
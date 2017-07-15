// Copyright 2017 Sean Gillespie.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.
use log::{self, Record, Level, Metadata, LevelFilter};
use std::sync::atomic::{self, AtomicBool, Ordering};

static DEBUG_ENABLED : AtomicBool = atomic::ATOMIC_BOOL_INIT;

struct UciLogger;

impl log::Log for UciLogger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        // the thinking behind this is:
        //  debug - extremely verbose, debug-build-only
        //  info  - verbose, "debug on" only
        //  warn/error - indicate something is wrong and always on
        if DEBUG_ENABLED.load(Ordering::Relaxed) {
            true
        } else {
            metadata.level() <= Level::Warn
        }
    }

    fn log(&self, record: &Record) {
        if self.enabled(record.metadata()) {
            println!("info string {}: {}", record.level(), record.args());
        }
    }
}

pub fn initialize() {
    let filter = LevelFilter::Info;
    let logger = Box::new(UciLogger);
    log::set_logger(logger, filter);
}

pub fn debug_enable() {
    DEBUG_ENABLED.store(true, Ordering::Release);
}

pub fn debug_disable() {
    DEBUG_ENABLED.store(false, Ordering::Release);
}
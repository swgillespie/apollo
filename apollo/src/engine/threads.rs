// Copyright 2017 Sean Gillespie.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.
use std::thread::{self, JoinHandle, Builder};
use parking_lot::RwLock;

struct ThreadStore {
    threads: RwLock<Vec<JoinHandle<()>>>,
}

impl ThreadStore {
    pub fn new() -> ThreadStore {
        ThreadStore { threads: RwLock::new(vec![]) }
    }

    pub fn with_main_thread<F, R>(&self, mut fun: F) -> R
        where F: FnMut(&JoinHandle<()>) -> R
    {
        let lock = self.threads.read();
        let main = lock.get(0).expect("with_main_thread called before initialization");
        return fun(main);
    }

    pub fn spawn<F>(&self, name: &'static str, init: F) 
        where F: FnOnce() + Send + 'static
    {
        let handle = Builder::new()
            .name(name.to_string())
            .spawn(init)
            .expect("failed to spawn thread");

        let mut lock = self.threads.write();
        lock.push(handle);
    }
}

lazy_static! {
    static ref STORE : ThreadStore = ThreadStore::new();
}

pub fn initialize() {
    // spawn the "main" search thread.
    STORE.spawn("search-main", || {
        search_thread_thunk();
    });
}

fn search_thread_thunk() {
    let this_thread = thread::current();
    info!("starting thread: {}", this_thread.name().unwrap());
    loop {
    }
}
// Copyright 2017 Sean Gillespie.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.
use std::f64;
use std::thread::{self, JoinHandle, Builder};
use std::sync::atomic::{AtomicBool, Ordering, AtomicUsize};
use std::sync::Arc;
use parking_lot::{RwLock, Condvar, Mutex};
use apollo_engine::Move;
use cancellation::{CancellationTokenSource, CancellationToken};
use engine::{self, SearchRequest};
use search;

struct ThreadStore {
    threads: RwLock<Vec<JoinHandle<()>>>,
}

impl ThreadStore {
    pub fn new() -> ThreadStore {
        ThreadStore { threads: RwLock::new(vec![]) }
    }

    pub fn spawn<F>(&self, name: &'static str, init: F)
        where F: FnOnce() + Send + 'static
    {
        let handle =
            Builder::new().name(name.to_string()).spawn(init).expect("failed to spawn thread");

        let mut lock = self.threads.write();
        lock.push(handle);
    }

    pub fn shutdown(&self) {
        debug!("thread store joining for shutdown");
        SHOULD_TERMINATE.store(true, Ordering::Release);
        // wake up everyone so they can head to a point
        // where they can exit
        WORK_CONDVAR.notify_all();
        let mut threads = self.threads.write();
        for thread in threads.drain(0..) {
            thread.join().unwrap();
        }

        debug!("thread store shutdown complete");
    }
}

lazy_static! {
    static ref STORE : ThreadStore = ThreadStore::new();
    static ref CANCEL : RwLock<CancellationTokenSource> = RwLock::new(CancellationTokenSource::new());
    static ref WORK_MUTEX : Mutex<Arc<SearchRequest>> = Mutex::new(Arc::new(SearchRequest::new()));
    static ref RESULTS_MUTEX : Mutex<Vec<(f64, Move)>> = Mutex::new(vec![]);
}

static WORK_CONDVAR: Condvar = Condvar::new();
static SHOULD_TERMINATE: AtomicBool = AtomicBool::new(false);
static RUNNING_THREADS: AtomicUsize = AtomicUsize::new(0);
static ALL_THREADS_DONE_MUTEX: Mutex<()> = Mutex::new(());
static ALL_THREADS_DONE_CONDVAR: Condvar = Condvar::new();

pub fn initialize() {
    // spawn the "main" search thread.
    STORE.spawn("search-main", || { search_thread_thunk(); });
}

pub fn shutdown() {
    STORE.shutdown();
}

pub fn request_search(req: Arc<SearchRequest>) {
    // set up the work state for this search session
    {
        let mut work = WORK_MUTEX.lock();
        let mut results = RESULTS_MUTEX.lock();
        *work = req;
        results.clear();
        // CTSes can't be reset, so we'll create a new one for every search.
        *CANCEL.write() = CancellationTokenSource::new();
        WORK_CONDVAR.notify_all();
    }

    // assert that one thread is running before returning
    while RUNNING_THREADS.load(Ordering::SeqCst) == 0 {}
}

pub fn cancel_search() {
    CANCEL.read().cancel();
}

/// Blocks the calling thread until results are ready for the requested
/// search.
pub fn request_results() -> (f64, Move) {
    while RUNNING_THREADS.load(Ordering::SeqCst) > 0 {
        info!("aux thread dropping into wait on search results");
        let mut lock = ALL_THREADS_DONE_MUTEX.lock();
        ALL_THREADS_DONE_CONDVAR.wait(&mut lock);
        info!("aux thread awoken");
    }

    info!("aux thread awoken, calculating results");
    // all done - let's go.
    let results = RESULTS_MUTEX.lock();
    // f64 doesn't implement ord... :<
    let (mut max_score, mut max_move) = (-f64::INFINITY, Move::null());
    for &(score, mov) in results.iter() {
        if score > max_score {
            max_score = score;
            max_move = mov;
        }
    }

    (max_score, max_move)
}

fn search_thread_thunk() {
    let this_thread = thread::current();
    let name = this_thread.name().unwrap();
    info!("starting thread: {}", name);
    RUNNING_THREADS.fetch_add(1, Ordering::SeqCst);
    loop {
        info!("thread `{}` going to sleep, waiting for work", name);
        if SHOULD_TERMINATE.load(Ordering::Acquire) {
            break;
        }

        let request = {
            let mut lock = WORK_MUTEX.lock();
            if RUNNING_THREADS.fetch_sub(1, Ordering::SeqCst) == 1 {
                info!("thread `{}` last to shut down, waking up aux thread", name);
                let _lock = ALL_THREADS_DONE_MUTEX.lock();
                ALL_THREADS_DONE_CONDVAR.notify_all();
            }

            WORK_CONDVAR.wait(&mut lock);
            RUNNING_THREADS.fetch_add(1, Ordering::SeqCst);
            lock.clone()
        };

        info!("thread `{}` awoken for search work", name);
        if SHOULD_TERMINATE.load(Ordering::Acquire) {
            break;
        }

        let cts = CANCEL.read();
        let pos = engine::CURRENT_POS.read().clone();
        let (score, mov) = search::search(&pos, request.depth, cts.token());
        info!("thread `{}` reports move `{}` (score: {})",
              name,
              mov.as_uci(),
              score);
        let mut results = RESULTS_MUTEX.lock();
        results.push((score, mov));
    }


    info!("thread `{}` terminating", name);
    if RUNNING_THREADS.fetch_sub(1, Ordering::SeqCst) == 1 {
        let _lock = ALL_THREADS_DONE_MUTEX.lock();
        ALL_THREADS_DONE_CONDVAR.notify_all();
    }
}
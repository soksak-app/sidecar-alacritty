#[allow(dead_code)]
#[path = "../src/engine.rs"]
mod engine;

use engine::AlacrittyEngine;
use soksak_sidecar_alacritty_core::Engine;
use std::alloc::{GlobalAlloc, Layout, System};
use std::sync::Mutex;
use std::sync::atomic::{AtomicUsize, Ordering};

/// The counters are shared by the tests of this binary, so each test that measures holds this lock
/// while it runs.
static MEASURING: Mutex<()> = Mutex::new(());

/// Counts the live bytes, their peak and the allocations of this test binary, which runs these
/// tests, so the numbers state what the engine allocates.
struct Counting;

static LIVE: AtomicUsize = AtomicUsize::new(0);
static PEAK: AtomicUsize = AtomicUsize::new(0);
static ALLOCATIONS: AtomicUsize = AtomicUsize::new(0);

// SAFETY: every call forwards to the system allocator with the same arguments.
unsafe impl GlobalAlloc for Counting {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let live = LIVE.fetch_add(layout.size(), Ordering::Relaxed) + layout.size();
        PEAK.fetch_max(live, Ordering::Relaxed);
        ALLOCATIONS.fetch_add(1, Ordering::Relaxed);
        unsafe { System.alloc(layout) }
    }
    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        LIVE.fetch_sub(layout.size(), Ordering::Relaxed);
        unsafe { System.dealloc(ptr, layout) }
    }
}

#[global_allocator]
static ALLOCATOR: Counting = Counting;

const CHUNK_BYTES: usize = 64 * 1024;
const CHUNKS: usize = 16;

/// A chunk that recycles the rows of the history allocates no row per line.
const FLOOD_ALLOCATIONS_PER_CHUNK: usize = 64;

/// The held sequence and the buffer of the parser each grow by doubling up to the bound of a
/// sequence, about 1.4 MiB.
const HELD_SEQUENCE_PEAK_BOUND: usize = 6 * 1024 * 1024;

/// An OSC that never ends holds at most the bound of a sequence.
#[test]
fn an_osc_that_never_ends_holds_at_most_the_bound_of_a_sequence() {
    let _measuring = MEASURING.lock().expect("measurement lock is not poisoned");
    let mut engine = AlacrittyEngine::new();
    let chunk = vec![b'x'; CHUNK_BYTES];
    let live = LIVE.load(Ordering::Relaxed);
    PEAK.store(live, Ordering::Relaxed);
    engine.feed(b"\x1b]0;");
    for _ in 0..256 {
        engine.feed(&chunk);
    }
    let peak = PEAK.load(Ordering::Relaxed) - live;
    println!("an OSC of 16 MiB: peak {peak} bytes");
    assert!(peak <= HELD_SEQUENCE_PEAK_BOUND, "peak {peak} bytes");
}

#[test]
fn a_flood_at_the_history_limit_allocates_no_rows_per_chunk() {
    let _measuring = MEASURING.lock().expect("measurement lock is not poisoned");
    let mut engine = AlacrittyEngine::new();
    let chunk = b"y\n".repeat(CHUNK_BYTES / 2);
    // The history fills first, so the flood below only recycles rows.
    for _ in 0..2 {
        engine.feed(&b"y\n".repeat(30_000));
    }
    let live = LIVE.load(Ordering::Relaxed);
    PEAK.store(live, Ordering::Relaxed);
    let allocations = ALLOCATIONS.load(Ordering::Relaxed);
    for _ in 0..CHUNKS {
        engine.feed(&chunk);
    }
    let allocated = ALLOCATIONS.load(Ordering::Relaxed) - allocations;
    println!("flood of {CHUNKS} chunks of {CHUNK_BYTES} bytes: {allocated} allocations");
    assert!(
        allocated <= CHUNKS * FLOOD_ALLOCATIONS_PER_CHUNK,
        "{allocated} allocations for {CHUNKS} chunks"
    );
}

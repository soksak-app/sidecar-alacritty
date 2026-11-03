#[allow(dead_code)]
#[path = "../src/engine.rs"]
mod engine;

use engine::AlacrittyEngine;
use soksak_sidecar_alacritty_core::Engine;
use std::alloc::{GlobalAlloc, Layout, System};
use std::sync::atomic::{AtomicUsize, Ordering};

/// Counts the live bytes, their peak and the allocations of this test binary, which runs these
/// tests, so the numbers state what the engine allocates.
struct Counting;

static LIVE: AtomicUsize = AtomicUsize::new(0);
static PEAK: AtomicUsize = AtomicUsize::new(0);

// SAFETY: every call forwards to the system allocator with the same arguments.
unsafe impl GlobalAlloc for Counting {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let live = LIVE.fetch_add(layout.size(), Ordering::Relaxed) + layout.size();
        PEAK.fetch_max(live, Ordering::Relaxed);
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

/// The held sequence and the buffer of the parser each grow by doubling up to the bound of a
/// sequence, about 1.4 MiB.
const HELD_SEQUENCE_PEAK_BOUND: usize = 6 * 1024 * 1024;

/// An OSC that never ends holds at most the bound of a sequence.
#[test]
fn an_osc_that_never_ends_holds_at_most_the_bound_of_a_sequence() {
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

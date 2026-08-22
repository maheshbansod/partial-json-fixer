//! Allocation-counting harness: reports heap allocations per call for each
//! public entry point on every sample input. Run with:
//!
//! ```sh
//! cargo bench --bench allocations
//! ```
//!
//! This is a plain `main()` rather than a criterion benchmark: allocation
//! counting must be exact, and a sampling framework's own bookkeeping would
//! pollute the counters. The global allocator below tallies calls; we reset,
//! invoke the function once, drop the result, and diff.

use std::alloc::{GlobalAlloc, Layout, System};
use std::sync::atomic::{AtomicUsize, Ordering::Relaxed};

mod common;

use common::{boundary_prefix, cases};
use partial_json_fixer::{fix_json, fix_json_parse};

static ALLOC_COUNT: AtomicUsize = AtomicUsize::new(0);
static ALLOC_BYTES: AtomicUsize = AtomicUsize::new(0);

struct CountingAllocator;

unsafe impl GlobalAlloc for CountingAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        ALLOC_COUNT.fetch_add(1, Relaxed);
        ALLOC_BYTES.fetch_add(layout.size(), Relaxed);
        System.alloc(layout)
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        System.dealloc(ptr, layout)
    }

    unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
        ALLOC_COUNT.fetch_add(1, Relaxed);
        ALLOC_BYTES.fetch_add(new_size.saturating_sub(layout.size()), Relaxed);
        System.realloc(ptr, layout, new_size)
    }

    unsafe fn alloc_zeroed(&self, layout: Layout) -> *mut u8 {
        ALLOC_COUNT.fetch_add(1, Relaxed);
        ALLOC_BYTES.fetch_add(layout.size(), Relaxed);
        System.alloc_zeroed(layout)
    }
}

#[global_allocator]
static GLOBAL: CountingAllocator = CountingAllocator;

fn snapshot() -> (usize, usize) {
    (ALLOC_COUNT.load(Relaxed), ALLOC_BYTES.load(Relaxed))
}

fn measure<R>(f: impl FnOnce() -> R) -> (usize, usize) {
    // Drops only trigger dealloc (which we don't count), so it's safe to let
    // the result die after taking the counter diff.
    let before = snapshot();
    let _result = f();
    let after = snapshot();
    (after.0 - before.0, after.1 - before.1)
}

fn main() {
    println!(
        "{:<22} {:<18} {:>10} {:>14}",
        "case", "input", "allocs", "alloc bytes"
    );
    println!("{}", "-".repeat(68));

    for case in cases() {
        let mut inputs: Vec<(String, String)> = vec![("full".into(), case.full.clone())];
        for frac in [0.1, 0.5, 0.9] {
            inputs.push((
                format!("pct-{}", (frac * 100.0) as u32),
                boundary_prefix(&case.full, frac).to_string(),
            ));
        }

        for (label, input) in &inputs {
            let (a, b) = measure(|| fix_json(input));
            println!(
                "{:<22} {:<18} {:>10} {:>14}",
                format!("fix_json/{}", case.name),
                label,
                a,
                b
            );
        }
        for (label, input) in &inputs {
            let (a, b) = measure(|| fix_json_parse(input));
            println!(
                "{:<22} {:<18} {:>10} {:>14}",
                format!("fix_json_parse/{}", case.name),
                label,
                a,
                b
            );
        }
        // Display round-trip: parse once outside the measurement, then count
        // what rendering costs.
        if let Ok(value) = fix_json_parse(&case.full) {
            let (a, b) = measure(|| format!("{value}"));
            println!(
                "{:<22} {:<18} {:>10} {:>14}",
                format!("display/{}", case.name),
                "full",
                a,
                b
            );
        }
        println!("{}", "-".repeat(68));
    }
}

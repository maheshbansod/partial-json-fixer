//! Throughput benchmarks for the two public entry points.
//!
//! Run with:
//!
//! ```sh
//! cargo bench --bench json
//! ```
//!
//! `fix_json` (string-based repair) is compared head-to-head with
//! `fix_json_parse` (tokenize + parse into a borrowed AST) across partial
//! prefixes of each sample document. `serde_json/from_str-full` is included
//! as a reference point: what parsing fully valid JSON costs, to show how
//! much overhead the "partial" handling adds.

use std::hint::black_box;

use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};

mod common;

use common::{boundary_prefix, cases};
use partial_json_fixer::{fix_json, fix_json_parse};

fn bench_fix_json(c: &mut Criterion) {
    let mut group = c.benchmark_group("fix_json");
    for case in cases() {
        // Full document as a sanity baseline, then partial prefixes at
        // increasing completion fractions (the streaming use case).
        for (label, input) in prefix_inputs(&case.full) {
            group.throughput(Throughput::Bytes(input.len() as u64));
            group.bench_with_input(
                BenchmarkId::new(case.name, &label),
                &input,
                |b, input| b.iter(|| fix_json(black_box(input))),
            );
        }
    }
    group.finish();
}

fn bench_fix_json_parse(c: &mut Criterion) {
    let mut group = c.benchmark_group("fix_json_parse");
    for case in cases() {
        for (label, input) in prefix_inputs(&case.full) {
            group.throughput(Throughput::Bytes(input.len() as u64));
            group.bench_with_input(
                BenchmarkId::new(case.name, &label),
                &input,
                |b, input| b.iter(|| fix_json_parse(black_box(input))),
            );
        }
    }
    group.finish();
}

/// Rendering the parsed AST back to a string — exercises the Display impls,
/// which currently build intermediate Vec<String>s per member.
fn bench_display(c: &mut Criterion) {
    let mut group = c.benchmark_group("display");
    for case in cases() {
        if let Ok(value) = fix_json_parse(&case.full) {
            group.throughput(Throughput::Bytes(case.full.len() as u64));
            group.bench_function(case.name, |b| {
                b.iter(|| format!("{}", black_box(&value)))
            });
        }
    }
    group.finish();
}

/// serde_json parsing the *complete* document, as an upper-bound reference.
fn bench_serde_reference(c: &mut Criterion) {
    let mut group = c.benchmark_group("serde_json");
    for case in cases() {
        group.throughput(Throughput::Bytes(case.full.len() as u64));
        group.bench_function(case.name, |b| {
            b.iter(|| serde_json::from_str::<serde_json::Value>(black_box(&case.full)))
        });
    }
    group.finish();
}

/// (benchmark label, input) pairs: the full doc plus prefixes at 10/50/90%.
fn prefix_inputs(full: &str) -> Vec<(String, String)> {
    let mut inputs = vec![("full".to_string(), full.to_string())];
    for frac in [0.1, 0.5, 0.9] {
        let cut = boundary_prefix(full, frac);
        inputs.push((format!("pct-{}", (frac * 100.0) as u32), cut.to_string()));
    }
    inputs
}

criterion_group!(
    benches,
    bench_fix_json,
    bench_fix_json_parse,
    bench_display,
    bench_serde_reference,
);
criterion_main!(benches);

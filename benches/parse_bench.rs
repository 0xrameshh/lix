use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn bench_parse_minimal(c: &mut Criterion) {
    let fixture = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join("claude_minimal.jsonl");
    c.bench_function("parse_minimal", |b| {
        b.iter(|| {
            let events = lix::read_all_events(black_box(&fixture)).unwrap();
            black_box(events);
        });
    });
}

fn bench_parse_toolcall(c: &mut Criterion) {
    let fixture = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join("claude_toolcall.jsonl");
    c.bench_function("parse_toolcall", |b| {
        b.iter(|| {
            let events = lix::read_all_events(black_box(&fixture)).unwrap();
            black_box(events);
        });
    });
}

criterion_group!(benches, bench_parse_minimal, bench_parse_toolcall);
criterion_main!(benches);

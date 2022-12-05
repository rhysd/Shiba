use criterion::{criterion_group, criterion_main, Criterion};
use shiba_bench::asset;
use shiba_preview::{MarkdownParseTarget, MarkdownParser, RawMessageWriter};

#[inline]
fn run(source: String) {
    let target = MarkdownParseTarget::new(source, None);
    let parser = MarkdownParser::new(&target, None, ());
    let mut buf = String::new();
    let () = parser.write_to(&mut buf).unwrap();
    assert!(!buf.is_empty());
}

fn encode(c: &mut Criterion) {
    c.bench_function("markdown::small", |b| {
        let source = asset("example.md");
        b.iter(move || run(source.clone()));
    });
    c.bench_function("markdown::middle", |b| {
        let source = asset("actionlint.md");
        b.iter(move || run(source.clone()));
    });
    c.bench_function("markdown::large", |b| {
        let source = asset("rust_releases.md");
        b.iter(move || run(source.clone()));
    });
}

criterion_group!(benches, encode);
criterion_main!(benches);

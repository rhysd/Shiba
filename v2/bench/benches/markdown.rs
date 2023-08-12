use criterion::{criterion_group, criterion_main, Criterion};
use shiba_bench::asset;
use shiba_preview::{MarkdownContent, MarkdownParser, RawMessageWriter};

#[inline]
fn run(source: String, offset: Option<usize>) {
    let target = MarkdownContent::new(source, None);
    let parser = MarkdownParser::new(&target, offset, ());
    let mut buf = Vec::new();
    let () = parser.write_to(&mut buf).unwrap();
    let buf = String::from_utf8(buf).unwrap();
    assert!(!buf.is_empty());
}

fn parse(c: &mut Criterion) {
    let small = asset("example.md");
    c.bench_function("markdown::small", |b| {
        b.iter(|| run(small.to_string(), None));
    });
    c.bench_function("markdown::middle", |b| {
        let source = asset("actionlint.md");
        b.iter(move || run(source.clone(), None));
    });
    c.bench_function("markdown::large", |b| {
        let source = asset("rust_releases.md");
        b.iter(move || run(source.clone(), None));
    });
    c.bench_function("markdown::offset", |b| {
        let offset = Some(small.len() / 2);
        b.iter(|| run(small.to_string(), offset));
    });
}

criterion_group!(benches, parse);
criterion_main!(benches);

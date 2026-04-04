use criterion::measurement::WallTime;
use criterion::{
    BenchmarkGroup, BenchmarkId, Criterion, Throughput, criterion_group, criterion_main,
};
use shiba_bench::asset;
use shiba_preview::bench::{
    DisplayText, History, MarkdownContent, MarkdownParser, RawMessageWriter, SearchMatcher,
    modified_offset, modified_offset_scalar,
};
use std::hint::black_box;
use std::io::sink;
use std::path::PathBuf;

fn markdown_parse(g: &mut BenchmarkGroup<'_, WallTime>) {
    fn run(source: String, offset: Option<usize>) {
        let target = MarkdownContent::new(source, None);
        let parser = MarkdownParser::new(&target, offset, ());
        let mut buf = Vec::new();
        let () = parser.write_to(&mut buf).unwrap();
        let buf = String::from_utf8(buf).unwrap();
        assert!(!buf.is_empty());
    }

    let small = asset("example.md");
    g.throughput(Throughput::Bytes(small.len() as _));
    g.bench_function(BenchmarkId::new("parse", "small"), |b| {
        b.iter(|| run(small.clone(), None));
    });

    let middle = asset("actionlint.md");
    g.throughput(Throughput::Bytes(middle.len() as _));
    g.bench_function(BenchmarkId::new("parse", "middle"), |b| {
        b.iter(|| run(middle.clone(), None));
    });

    let large = asset("the_book.md");
    g.throughput(Throughput::Bytes(large.len() as _));
    g.bench_function(BenchmarkId::new("parse", "large"), |b| {
        b.iter(|| run(large.clone(), None));
    });

    g.throughput(Throughput::Bytes(small.len() as _));
    g.bench_function(BenchmarkId::new("parse", "offset"), |b| {
        let offset = Some(small.len() / 2);
        b.iter(|| run(small.clone(), offset));
    });
}

fn markdown_search(g: &mut BenchmarkGroup<'_, WallTime>) {
    struct Case<'a> {
        name: &'a str,
        matcher: SearchMatcher,
        query: &'a str,
    }

    let source = asset("actionlint.md");
    g.throughput(Throughput::Bytes(source.len() as u64));

    let content = MarkdownContent::new(source, None);
    let parser = MarkdownParser::new(&content, None, ());
    let text: DisplayText = parser.write_to(sink()).unwrap();

    let cases = [
        Case { name: "case_sensitive", matcher: SearchMatcher::CaseSensitive, query: "a" },
        Case { name: "case_sensitive", matcher: SearchMatcher::CaseSensitive, query: "actionlint" },
        Case { name: "case_insensitive", matcher: SearchMatcher::CaseInsensitive, query: "a" },
        Case {
            name: "case_insensitive",
            matcher: SearchMatcher::CaseInsensitive,
            query: "ACTIONLINT",
        },
        Case { name: "regex", matcher: SearchMatcher::CaseSensitiveRegex, query: "a" },
        Case {
            name: "regex",
            matcher: SearchMatcher::CaseSensitiveRegex,
            query: "\\baction\\w+\\b",
        },
    ];

    for case in cases {
        let param = format!("{}_{}", case.name, case.query.chars().count());
        g.bench_with_input(BenchmarkId::new("search", param), &case, |b, c| {
            b.iter(|| {
                let matches = text.search(black_box(c.query), black_box(c.matcher)).unwrap();
                assert!(matches.len() > 0);
                let tokenizer = matches.tokenizer(Some(0)).unwrap();
                let mut buf = Vec::new();
                let () = MarkdownParser::new(&content, None, tokenizer).write_to(&mut buf).unwrap();
                assert!(!buf.is_empty());
            });
        });
    }
}

fn markdown(c: &mut Criterion) {
    let mut g = c.benchmark_group("markdown");
    markdown_parse(&mut g);
    markdown_search(&mut g);
    g.finish();
}

fn history(c: &mut Criterion) {
    let mut g = c.benchmark_group("history");

    let paths: Vec<_> = (0..10000).map(|i| PathBuf::from(format!("path/to/{i}.md"))).collect();
    g.throughput(Throughput::Elements(10000));
    g.bench_function(BenchmarkId::new("push", "new"), |b| {
        b.iter(|| {
            let mut history = History::new(10000);
            for path in &paths {
                history.push(path.clone());
                assert_eq!(history.current(), Some(path.as_path()));
            }
        });
    });

    let paths: Vec<_> = (0..10000).map(|i| PathBuf::from(format!("path/to/{i}.md"))).collect();
    g.throughput(Throughput::Elements(10000));
    g.bench_function(BenchmarkId::new("push", "discard"), |b| {
        b.iter(|| {
            let mut history = History::new(100);
            for path in &paths {
                history.push(path.clone());
                assert_eq!(history.current(), Some(path.as_path()));
            }
        });
    });

    let paths: Vec<_> = (0..100).map(|i| PathBuf::from(format!("path/to/{i}.md"))).collect();
    g.throughput(Throughput::Elements(10000));
    g.bench_function(BenchmarkId::new("push", "existing"), |b| {
        b.iter(|| {
            let mut history = History::new(100);
            for _ in 0..100 {
                for path in &paths {
                    history.push(path.clone());
                    assert_eq!(history.current(), Some(path.as_path()));
                }
            }
        });
    });

    g.finish();
}

fn bytes(c: &mut Criterion) {
    fn with_diff_at(len: usize, pos: usize) -> (Vec<u8>, Vec<u8>) {
        assert!(pos < len);
        let left = vec![b'a'; len];
        let mut right = left.clone();
        right[pos] = b'b';
        (left, right)
    }

    let mut g = c.benchmark_group("bytes");
    let cases = [
        ("48_end", with_diff_at(48, 47)),
        ("512_middle", with_diff_at(512, 256)),
        ("512_end", with_diff_at(512, 511)),
        ("4k_begin", with_diff_at(4 * 1024, 0)),
        ("4k_middle", with_diff_at(4 * 1024, 2 * 1024)),
        ("4k_end", with_diff_at(4 * 1024, 4 * 1024 - 1)),
        ("4k_equal", (vec![b'a'; 4 * 1024], vec![b'a'; 4 * 1024])),
        ("64k_middle", with_diff_at(64 * 1024, 32 * 1024)),
        ("64k_end", with_diff_at(64 * 1024, 64 * 1024 - 1)),
        ("512k_middle", with_diff_at(512 * 1024, 256 * 1024)),
        ("512k_end", with_diff_at(512 * 1024, 512 * 1024 - 1)),
    ];

    for (name, (left, right)) in &cases {
        g.throughput(Throughput::Bytes(left.len().min(right.len()) as u64));

        g.bench_with_input(
            BenchmarkId::new("modified_offset", format!("scalar_{name}")),
            &(left, right),
            |b, &(left, right)| {
                b.iter(|| {
                    let ret = modified_offset_scalar(black_box(left), black_box(right));
                    black_box(ret);
                });
            },
        );

        g.bench_with_input(
            BenchmarkId::new("modified_offset", format!("simd_{name}")),
            &(left, right),
            |b, &(left, right)| {
                b.iter(|| {
                    let ret = modified_offset(black_box(left), black_box(right));
                    black_box(ret);
                });
            },
        );
    }

    g.finish();
}

criterion_group!(benches, markdown, history, bytes);
criterion_main!(benches);

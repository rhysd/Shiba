use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use shiba_preview::bench::{modified_offset_scalar, modified_offset_simd};
use std::hint::black_box;

fn with_diff_at(len: usize, pos: usize) -> (Vec<u8>, Vec<u8>) {
    assert!(pos < len);

    let left = vec![b'a'; len];
    let mut right = left.clone();
    right[pos] = b'b';
    (left, right)
}

fn bench_case(c: &mut Criterion, group: &str, name: &str, left: &[u8], right: &[u8]) {
    let mut g = c.benchmark_group(group);
    g.throughput(Throughput::Bytes(left.len().min(right.len()) as u64));

    g.bench_with_input(BenchmarkId::new("scalar", name), &(left, right), |b, &(left, right)| {
        b.iter(|| {
            let ret = modified_offset_scalar(black_box(left), black_box(right));
            black_box(ret);
        });
    });

    g.bench_with_input(BenchmarkId::new("simd", name), &(left, right), |b, &(left, right)| {
        b.iter(|| {
            let ret = modified_offset_simd(black_box(left), black_box(right));
            black_box(ret);
        });
    });

    g.finish();
}

fn bytes(c: &mut Criterion) {
    let cases = [
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
        bench_case(c, "bytes", name, left, right);
    }
}

criterion_group!(benches, bytes);
criterion_main!(benches);

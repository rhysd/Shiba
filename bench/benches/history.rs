use criterion::{criterion_group, criterion_main, Criterion};
use shiba_preview::bench::History;
use std::path::PathBuf;

fn push(c: &mut Criterion) {
    // Push 10000 new history items with discarding/ignoring nothing
    c.bench_function("history::push::new", |b| {
        let paths: Vec<_> = (0..10000).map(|i| PathBuf::from(format!("path/to/{i}.md"))).collect();
        b.iter(|| {
            let mut history = History::new(10000);
            for path in &paths {
                history.push(path.clone());
                assert_eq!(history.current(), Some(path.as_path()));
            }
        });
    });

    // Push 10000 new history items discarding 9900 items since the number of max recent files is 100
    c.bench_function("history::push::discard", |b| {
        let paths: Vec<_> = (0..10000).map(|i| PathBuf::from(format!("path/to/{i}.md"))).collect();
        b.iter(|| {
            let mut history = History::new(100);
            for path in &paths {
                history.push(path.clone());
                assert_eq!(history.current(), Some(path.as_path()));
            }
        });
    });

    // Push 10000 new history items ignoring 9900 items since they are already in the history
    c.bench_function("history::push::existing", |b| {
        let paths: Vec<_> = (0..100).map(|i| PathBuf::from(format!("path/to/{i}.md"))).collect();
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
}

criterion_group!(benches, push);
criterion_main!(benches);

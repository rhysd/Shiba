Benchmarks for Shiba using [Criterion.rs][criterion].

To run all benchmarks:

```sh
cargo bench --benches
```

To run specific benchmark suite:

```sh
cargo bench --bench markdown
```

To filter benchmarks:

```sh
cargo bench markdown::large
```

To compare benchmark results with [critcmp][]:

```sh
git checkout main
cargo bench -- --save-baseline base

git checkout your-feature
cargo bench -- --save-baseline change

critcmp base change
```

[criterion]: https://github.com/bheisler/criterion.rs
[critcmp]: https://github.com/BurntSushi/critcmp

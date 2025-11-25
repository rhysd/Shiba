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

Files in [assets/](./assets/):

- example.md
  - The example input on testing the markdown parser. Image paths were adjusted
  - [all.md](../src/markdown/testdata/all.md)
- actionlint.md
  - The document of actionlint checks
  - https://github.com/rhysd/actionlint/blob/main/docs/checks.md
- the_book.md
  - 'The Rust Programming Language' book (Apache 2.0)
  - An amalgam of all markdown files in [the src directory](https://github.com/rust-lang/book/tree/main/src)
    ```sh
    cd /path/to/book && cat src/*.md > the_book.md
    ```

[criterion]: https://github.com/bheisler/criterion.rs
[critcmp]: https://github.com/BurntSushi/critcmp

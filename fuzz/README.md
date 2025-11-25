Fuzzing tests for Shiba's Markdown processor.

## Prerequisites

- [cargo-fuzz][]
- Nightly Rust toolchain

## How to run

```sh
# See list of fuzzing test cases
cargo fuzz list
# Run fuzzing tests
cargo +nightly fuzz run parse_markdown
```

[cargo-fuzz]: https://github.com/rust-fuzz/cargo-fuzz

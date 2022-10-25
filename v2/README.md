Shiba v2
========
[![CI for v2][ci-badge]][ci]

[Shiba](https://github.com/rhysd/Shiba) v2 will be a complate rewrite of v1 and will drop several features to keep the application simple.

The next implementation will:

- drop the built-in linter feature since checking the source directly in text editor is more useful
- drop the HTML preview feature since it is actually not used
- be a single executable using native WebView instead of Electron since it makes the installation much easier and the application size much smaller
- use Rust for the main logic and use TypeScript with React for the view logic since Rust is faster and safer

[ci]: https://github.com/rhysd/Shiba/actions/workflows/ci.yml
[ci-badge]: https://github.com/rhysd/Shiba/actions/workflows/ci.yml/badge.svg

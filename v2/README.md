Shiba v2
========
[![CI for v2][ci-badge]][ci]

https://user-images.githubusercontent.com/823277/235185829-28a9242f-016e-44a1-bf0b-8e96225f8be8.mp4

:warning: WORK IN PROGRESS

[Shiba](https://github.com/rhysd/Shiba) v2 will be a complete rewrite of v1 and will drop several features to keep the application simple.

The next implementation will:

- drop the built-in linter feature since checking the source directly in text editor is more useful
- drop the HTML preview feature since it is actually not used
- be a single executable using native WebView instead of Electron since it makes the installation much easier and the application size much smaller
  - Since it uses platform-specific WebView implementation, some WebView runtime would be necessary as dependencies (WebView2 runtime on Windows, libwebkit2gtk on Linux)
- use Rust for the main logic and use TypeScript with React for the view logic since Rust is faster and safer

[ci]: https://github.com/rhysd/Shiba/actions/workflows/ci.yml
[ci-badge]: https://github.com/rhysd/Shiba/actions/workflows/ci.yml/badge.svg

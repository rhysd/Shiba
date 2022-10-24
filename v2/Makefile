TS_SRC := $(wildcard web/*.ts)
RS_SRC := $(wildcard src/*.rs)

node_modules:
	npm install

dist/bundle.js: node_modules $(TS_SRC)
	npm run ts
	npm run bundle

target/debug/rust-webview: $(RS_SRC)
	cargo build

target/release/rust-webview: $(RS_SRC)
	cargo build --release

build: dist/bundle.js target/debug/rust-webview

release: dist/bundle.js target/release/rust-webview

.PHONY: build release

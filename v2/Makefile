TS_SRC := $(wildcard web/*.ts)
RS_SRC := $(wildcard src/*.rs)
DEBUG_DIST := dist/bundle.js
RELEASE_DIST := dist/bundle.min.js dist/github-markdown.min.css dist/hljs-github.min.css

node_modules:
	npm install

$(DEBUG_DIST): node_modules $(TS_SRC)
	npm run check:ts
	npm run bundle

$(RELEASE_DIST): node_modules $(TS_SRC)
	npm run check:ts
	npm run release

target/debug/shiba: $(RS_SRC) $(DEBUG_DIST) dist/template.html
	cargo build

target/release/shiba: $(RS_SRC) $(RELEASE_DIST) dist/template.html
	cargo build --release

.PHONY: build release clean

build: target/debug/shiba
release: target/release/shiba
clean:
	rm -rf $(DEBUG_DIST) $(RELEASE_DIST) node_modules target

.DEFAULT_GOAL := build

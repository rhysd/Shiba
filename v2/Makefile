TS_SRC := $(wildcard web/*.ts web/*.tsx web/components/*.tsx) web/style.css web/index.html
RS_SRC := $(wildcard src/*.rs)
CSS := $(shell find -E src/assets -type f -regex .*\.css$ )

node_modules:
	npm install

src/assets/bundle.js src/assets/bundle.js.map: node_modules $(TS_SRC)
	npm run lint:ts
	npm run bundle

src/assets/bundle.min.js: node_modules $(TS_SRC)
	npm run lint:ts
	npm run release

target/debug/shiba: $(RS_SRC) src/assets/bundle.js web/index.html
	cargo build

target/release/shiba: $(RS_SRC) src/assets/bundle.min.js web/index.html
	cargo build --release

.PHONY: build release clean

build: target/debug/shiba
release: target/release/shiba
clean:
	rm -rf src/assets/**/*.css src/assets/*.js src/assets/*.html node_modules target

.DEFAULT_GOAL := build

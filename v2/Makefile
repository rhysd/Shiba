TS_SRC := $(wildcard web/*.ts web/*.tsx web/components/*.tsx)
RS_SRC := $(wildcard src/*.rs)
CSS := $(shell find -E src/assets -type f -regex .*\.css$ )

node_modules:
	npm install

$(CSS): node_modules
	npm run bundle:css

src/assets/bundle.js: node_modules $(TS_SRC)
	npm run lint:ts
	npm run bundle:js

src/assets/bundle.min.js: node_modules $(TS_SRC)
	npm run lint:ts
	npm run release:js

target/debug/shiba: $(RS_SRC) $(CSS) src/assets/bundle.js src/assets/index.html
	cargo build

target/release/shiba: $(RS_SRC) $(CSS) src/assets/bundle.min.js src/assets/index.html
	cargo build --release

.PHONY: build release clean

build: target/debug/shiba
release: target/release/shiba
clean:
	rm -rf src/assets/**/*.css src/assets/*.js node_modules target

.DEFAULT_GOAL := build

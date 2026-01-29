TS_SRC := $(wildcard ui/*.ts ui/*.tsx ui/components/*.tsx) ui/style.css ui/index.html
RS_SRC := $(wildcard src/*.rs src/markdown/*.rs src/wry/*.rs) build.rs
CSS := $(shell find -E src/assets -type f -regex .*\.css$ )
MAC_APP_ASSETS := assets/Shiba.app/Contents/Info.plist assets/Shiba.app/Contents/Resources/icon.icns

node_modules: package.json
	npm install

src/assets/bundle.js src/assets/bundle.js.map: node_modules $(TS_SRC)
	npm run lint:tsc
	npm run bundle

src/assets/bundle.min.js: node_modules $(TS_SRC)
	npm run lint:tsc
	npm run release

ui/mathjax_loader.ts: node_modules
	node ./scripts/mathjax.mjs

target/debug/shiba: $(RS_SRC) src/assets/bundle.js src/assets/index.html $(CSS)
	cargo build
target/debug/shiba.exe: $(RS_SRC) src/assets/bundle.js src/assets/index.html $(CSS) assets/icon.ico
	cargo build

target/release/shiba: $(RS_SRC) src/assets/bundle.min.js src/assets/index.html $(CSS)
	cargo build --release
target/release/shiba.exe: $(RS_SRC) src/assets/bundle.min.js src/assets/index.html $(CSS) assets/icon.ico
	cargo build --release

target/x86_64-apple-darwin/release/shiba: $(RS_SRC) src/assets/bundle.min.js src/assets/index.html $(CSS)
	cargo build --release --target=x86_64-apple-darwin
target/aarch64-apple-darwin/release/shiba: $(RS_SRC) src/assets/bundle.min.js src/assets/index.html $(CSS)
	cargo build --release --target=aarch64-apple-darwin
Shiba.app: target/x86_64-apple-darwin/release/shiba target/aarch64-apple-darwin/release/shiba $(MAC_APP_ASSETS)
	bash ./scripts/gen_macos_app.bash
Shiba.dmg: Shiba.app README.md LICENSE
	bash ./scripts/gen_macos_dmg.bash

shiba.msi: target/release/shiba.exe assets/wix/shiba.wxs
	wix extension add WixToolset.UI.wixext WixToolset.Util.wixext
	wix build -arch "x64" -ext WixToolset.UI.wixext -ext WixToolset.Util.wixext -out shiba.msi assets/wix/shiba.wxs -define ShibaVersion=(yq .package.version Cargo.toml)

target/debian/shiba_%_amd64.deb: target/release/shiba assets/deb/shiba.desktop
	cargo deb --no-build --verbose
shiba_amd64.deb: target/debian/shiba_2.0.0-alpha.0_amd64.deb
	mv target/debian/shiba_*_amd64.deb shiba_amd64.deb

.PHONY: build release clean

build: target/debug/shiba
release: target/release/shiba
clean:
	rm -rf src/assets/**/*.css src/assets/*.js src/assets/*.html node_modules target Shiba.app Shiba.dmg shiba.msi shiba_amd64.deb

.DEFAULT_GOAL := build

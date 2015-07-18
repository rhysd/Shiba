BIN=./node_modules/.bin

run:
	$(BIN)/electron .

dep:
	@npm install
	@bower install

npm-publish:
	@mkdir -p npm-publish/resource/image
	@cp bower.json npm-publish/
	@cp package.json npm-publish/
	@cp -R resource/image/emoji npm-publish/resource/image/
	@cp -R src npm-publish/
	@cp -R static npm-publish/
	@cp -R bin npm-publish/
	@cp README.md npm-publish/
	@cd npm-publish && bower install --production
	@cd npm-publish && npm install --save electron-prebuilt
	@cd npm-publish && npm publish
	@rm -rf npm-publish

asar: clean
	@mkdir -p archive/resource/image
	@cp bower.json archive/
	@cp package.json archive/
	@cp -R resource/image/emoji archive/resource/image/
	@cp -R src archive/
	@cp -R static archive/
	@cd archive && npm install --production && bower install --production; cd ..
	@./node_modules/.bin/asar pack archive app.asar
	@rm -rf archive

clean:
	@rm -rf archive app.asar npm-publish

.PHONY: run dep npm-publish asar clean

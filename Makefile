BIN=./node_modules/.bin

run:
	$(BIN)/electron .

dep:
	@npm install
	@bower install

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
	@rm -rf archive
	@rm -f app.asar


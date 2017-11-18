#!/bin/bash

set -e

cd scripts
git clone --depth=1 https://github.com/github/gemoji.git
cd gemoji
bundle install --path=.bundle
bundle exec gemoji extract public/images/emoji
cd -
ruby prepare_emojis.rb
cp gemoji/tmp/*.png ../build/images/emoji/
cp gemoji/public/images/emoji/*.png ../build/images/emoji/

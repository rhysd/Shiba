#!/bin/bash

set -e -x -o pipefail

apt-get update -y -q

# Install libwebkit2gtk-4.1-dev for wry: https://github.com/tauri-apps/wry#platform-specific-notes
# Install libgtk-3-dev and libxdo-dev for muda: https://github.com/tauri-apps/muda#dependencies-linux-only
apt-get install -y -q --no-install-recommends libwebkit2gtk-4.1-dev libgtk-3-dev libxdo-dev

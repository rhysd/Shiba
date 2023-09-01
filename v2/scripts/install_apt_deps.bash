#!/bin/bash

set -e -x -o pipefail

apt-get update
apt-get install -y --no-install-recommends libwebkit2gtk-4.1-dev libgtk-3-dev libxdo-dev

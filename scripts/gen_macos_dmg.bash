#!/bin/bash

set -e -o pipefail

if [[ "$OSTYPE" != darwin* ]]; then
    echo 'ERROR: This script must be run on macOS' 1>&2
    exit 1
fi

if [ ! -d .git ]; then
    echo 'ERROR: This script must be run at repository root' 1>&2
    exit 1
fi

if [ ! -d ./Shiba.app ]; then
    echo 'ERROR: Shiba.app does not exist in the current directory' 1>&2
    exit 1
fi

echo "Generating Shiba.dmg in the current directory..."

temp_dir="$(mktemp -d)"
echo "Created a temporary directory ${temp_dir} for workspace"

osx_dir="${temp_dir}/osx"
mkdir -p "$osx_dir"

echo "Copying contents to the workspace..."

cp -R ./Shiba.app "$osx_dir"
cp ./LICENSE "$osx_dir"
cp ./README.md "$osx_dir"

echo "Generating .dmg archive with hdiutil..."
ln -sf /Applications "${osx_dir}/Applications"
hdiutil create ./Shiba.dmg -volname "Shiba" -fs HFS+ -srcfolder "$osx_dir" -ov -format UDZO

echo "Removing the temporary directory..."
rm -r "$temp_dir"

echo 'Done.'

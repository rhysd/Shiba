#!/bin/bash

set -e -o pipefail

if [[ "$OSTYPE" != darwin* ]]; then
    echo 'ERROR: This script must be run on macOS' 1>&2
    exit 1
fi

if [ ! -d ../.git ]; then
    echo 'ERROR: This script must be run at v2 directory' 1>&2
    exit 1
fi

echo "Generating Shiba.app in current directory..."
rm -rf ./Shiba.app

echo "Detecting version from Cargo.toml..."
pat='^version \= "([^"]+)"$'
while IFS=: read -r line; do
    if [[ "$line" =~ $pat ]]; then
        version="${BASH_REMATCH[1]}"
        break
    fi
done < ./Cargo.toml
if [ "$version" = "" ]; then
    echo 'ERROR: No version was found in Cargo.toml' 1>&2
    exit 1
fi

echo "Generating ./Shiba.app for version ${version}..."
cp -R ./assets/Shiba.app ./
sed -E -i '' "s/\\{\\{VERSION}}/${version}/" ./Shiba.app/Contents/Info.plist

echo "Copying 'shiba' release binary..."
if [ ! -x ./target/release/shiba ]; then
    echo "ERROR: ./target/release/shiba does not exist. Build it with 'cargo build --release'" 1>&2
    exit 1
fi
mkdir -p ./Shiba.app/Contents/MacOS/
cp ./target/release/shiba ./Shiba.app/Contents/MacOS/

echo 'Done.'

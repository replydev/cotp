#!/bin/bash

# Check if the version argument is provided
if [ $# -ne 1 ]; then
    echo "Usage: $0 <new_version>"
    exit 1
fi

# Update the version in Cargo.toml
new_version="$1"
sed -i "s/^version = \".*\"/version = \"$new_version\"/" Cargo.toml

# Update the same version in Cargo.lock (only under name = "cotp")
awk -v new_version="$new_version" '/^name = "cotp"/ { found=1 } found && /^version = "/ { if (found && !done) { sub(/".*"/, "\"" new_version "\""); done=1 } } 1' Cargo.lock > Cargo.lock.tmp
mv Cargo.lock.tmp Cargo.lock

echo "Version updated to $new_version in Cargo.toml and only under name = \"cotp\" in Cargo.lock."

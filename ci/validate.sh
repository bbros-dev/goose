#!/usr/bin/env bash
for file in book/src/*.md ; do
    echo Checking references in $file
    cargo run --quiet --bin link2print < $file > /dev/null
done
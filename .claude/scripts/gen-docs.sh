#!/bin/bash

# Install rustdoc-md if not already installed
if ! which rustdoc-md > /dev/null 2>&1; then
    cargo install rustdoc-md
fi

# Check the doc argument
if [ -z "$1" ]; then
    echo "Usage: $0 <crate>"
    exit 1
fi

CRATE=$1

# Generate docs
RUSTC_BOOTSTRAP=1 RUSTDOCFLAGS="-Z unstable-options --output-format json" cargo doc --workspace --all-features
rustdoc-md --path target/doc/$CRATE.json --output target/doc/$CRATE.md

echo "Docs generated for $CRATE (target/doc/$CRATE.md)"
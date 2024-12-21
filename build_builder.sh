#!/bin/bash

# Build the binary
cargo build --release

# Create builder directory and copy the binary
mkdir -p out/builder
cp target/release/debr out/builder/

# Create tar.gz archive
tar -czvf out/builder.tar.gz out/builder/

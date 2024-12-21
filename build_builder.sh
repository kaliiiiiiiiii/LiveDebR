#!/bin/bash

# Build the binary
cargo build --release

# Create builder directory and copy the binary
mkdir -p builder
cp target/release/LiveDebR builder/

# Create tar.gz archive
tar -czvf builder.tar.gz builder

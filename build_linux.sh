#!/bin/bash
cargo build --release
mkdir PhotoSelector
cp target/release/photo-selector PhotoSelector/PhotoSelector
chmod +x PhotoSelector/PhotoSelector
tar -czf PhotoSelector-Linux.tar.gz PhotoSelector 
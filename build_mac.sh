#!/bin/bash
cargo build --release
mkdir PhotoSelector
cp target/release/photo-selector PhotoSelector/PhotoSelector
chmod +x PhotoSelector/PhotoSelector
zip -r PhotoSelector-Mac.zip PhotoSelector 
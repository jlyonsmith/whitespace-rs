#!/bin/zsh
export RUSTFLAGS="-Zinstrument-coverage"
export LLVM_PROFILE_FILE="$(pwd)/scratch/$(whoami)-%p-%m.profraw"
cargo test
grcov . -s . --binary-path ./target/debug/ -t html --branch --ignore-not-existing -o ./target/debug/coverage/ --excl-start '^//\s*\{grcov-excl-start\}' --excl-stop '^//\s*\{grcov-excl-end\}'

#!/bin/zsh
export RUSTFLAGS="-Zinstrument-coverage"
export LLVM_PROFILE_FILE="./scratch/$(whoami)-%p-%m.profraw"
cargo test
grcov . -s . --binary-path ./target/debug/ -t html --branch --ignore-not-existing -o ./target/debug/coverage/

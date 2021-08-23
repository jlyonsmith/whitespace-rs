get-coverage:
  #!/usr/bin/env bash
  export RUSTFLAGS="-Zinstrument-coverage"
  export LLVM_PROFILE_FILE="$(pwd)/scratch/$(whoami)-%p-%m.profraw"
  cargo test --tests
  grcov . -s . --binary-path ./target/debug/ -t html --branch --ignore-not-existing -o ./target/debug/coverage/ --excl-start '^//\s*\{grcov-excl-start\}' --excl-stop '^//\s*\{grcov-excl-end\}'
  cp ./target/debug/coverage/coverage.json ./coverage.json

show-coverage:
  open target/debug/coverage/index.html

show-doc:
  cargo doc --open

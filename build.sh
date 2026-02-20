#!/usr/bin/env bash
set -euo pipefail
IFS=$'\n\t'

main() {
    cd ./rust
    cargo build --release
    sudo cp ./target/release/keyboard-rs /usr/bin
}

main "$@"

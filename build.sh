#!/usr/bin/env bash

# rustup install nightly
# rustup component add build-std
# rustup default nightly
cargo build -Z patch-in-config -Z build-std --target x86_64-unknown-uefi

#!/bin/bash
set -e

cargo fmt --all -- --check

cargo clippy --all-targets -- -D clippy::all -D clippy::pedantic
cd device
cargo clippy --all-targets --features libusb -- -D clippy::all -D clippy::pedantic
cd ..

cargo doc --no-deps

cargo test --all-targets
cargo test --doc

cd device
cargo test --all-targets --features libusb
cargo test --doc --features libusb
cd ..

cd gentl
cargo test --all-targets --features libusb
cd ..

cd cameleon
cargo test --all-targets --features libusb
cargo test --doc --features libusb
cd ..

echo "### all tests passed! ###"

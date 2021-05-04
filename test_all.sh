function exit_if_failed() {
    if [[ $1 != 0 ]]
    then
        exit $1
    fi
}

cargo fmt --all -- --check
exit_if_failed $?

## FIXME: The line should be removed when 1.52 is released, See https://github.com/rust-lang/rust-clippy/issues/4612 for more details.
find . | grep "\.rs$" | xargs touch

cargo clippy --all-targets -- -D clippy::all -D clippy::pedantic
exit_if_failed $?
cd device
cargo clippy --all-targets --features libusb -- -D clippy::all -D clippy::pedantic
exit_if_failed $?
cd ..

cargo doc --no-deps
exit_if_failed $?

cargo test --all-targets
exit_if_failed $?
cargo test --doc
exit_if_failed $?

cd device
cargo test --all-targets --features libusb
exit_if_failed $?
cargo test --doc --features libusb
exit_if_failed $?
cd ..

cd gentl
cargo test --all-targets --features libusb
exit_if_failed $?
cd ..

cd cameleon
cargo test --all-targets --features libusb
exit_if_failed $?
cargo test --doc --features libusb
exit_if_failed $?
cd ..

echo "### all tests passed! ###"

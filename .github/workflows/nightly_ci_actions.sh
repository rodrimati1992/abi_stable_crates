rustup override set nightly

cd "${TRAVIS_BUILD_DIR}/abi_stable/"
MIRI_NIGHTLY=nightly-$(curl -s https://rust-lang.github.io/rustup-components-history/x86_64-unknown-linux-gnu/miri)
echo "Installing latest nightly with Miri"
echo "$MIRI_NIGHTLY"
rustup set profile minimal
rustup default "$MIRI_NIGHTLY"
rustup override set "$MIRI_NIGHTLY"
rustup component add miri
cargo miri setup
env "MIRIFLAGS=-Zmiri-disable-isolation" cargo miri test --features testing

cd "${TRAVIS_BUILD_DIR}"
rm Cargo.lock
cd "${TRAVIS_BUILD_DIR}/abi_stable/"
cargo clean
cargo update -Z minimal-versions
cd "${TRAVIS_BUILD_DIR}/abi_stable/"
cargo build

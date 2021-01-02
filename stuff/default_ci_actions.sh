rustup override set ${{ matrix.rust }}
rustup install beta

cd "${GITHUB_WORKSPACE}/examples/0_modules_and_interface_types/impl/"
cargo +beta build

cd "${GITHUB_WORKSPACE}/examples/1_trait_objects/plugin_0/"
cargo +beta build

cd "${GITHUB_WORKSPACE}/examples/1_trait_objects/plugin_1/"
cargo +beta build

cd "${GITHUB_WORKSPACE}/examples/2_nonexhaustive/implementation/"
cargo +beta build

cd "${GITHUB_WORKSPACE}/testing/0/impl_0/"
cargo +beta build

cd "${GITHUB_WORKSPACE}/testing/1 - loading errors/impl_1/"
cargo +beta build

cd "${GITHUB_WORKSPACE}/testing/1 - loading errors/non_abi_stable_lib/"
cargo +beta build

cd "${GITHUB_WORKSPACE}/testing/version_compatibility/impl_0"
cargo +beta build

cd "${GITHUB_WORKSPACE}/"
rm Cargo.lock

cd "${GITHUB_WORKSPACE}/examples/0_modules_and_interface_types/impl/"
cargo check

cd "${GITHUB_WORKSPACE}/examples/1_trait_objects/plugin_0/"
cargo check

cd "${GITHUB_WORKSPACE}/examples/1_trait_objects/plugin_1/"
cargo check

cd "${GITHUB_WORKSPACE}/examples/2_nonexhaustive/implementation/"
cargo check

cd "${GITHUB_WORKSPACE}/testing/0/impl_0/"
cargo check

cd "${GITHUB_WORKSPACE}/testing/1 - loading errors/impl_1/"
cargo check

cd "${GITHUB_WORKSPACE}/testing/1 - loading errors/non_abi_stable_lib/"
cargo check

cd "${GITHUB_WORKSPACE}/abi_stable"
cargo check --no-default-features

cd "${GITHUB_WORKSPACE}/abi_stable_derive"
cargo test

cd "${GITHUB_WORKSPACE}/abi_stable"
cargo test --features "testing sabi_trait_examples"

cd "${GITHUB_WORKSPACE}/examples/0_modules_and_interface_types/impl/"
cargo test
cd "${GITHUB_WORKSPACE}/examples/0_modules_and_interface_types/user/"
cargo run -- run-tests

cd "${GITHUB_WORKSPACE}/examples/1_trait_objects/application/"
cargo run

cd "${GITHUB_WORKSPACE}/examples/2_nonexhaustive/implementation/"
cargo test
cd "${GITHUB_WORKSPACE}/examples/2_nonexhaustive/user/"
cargo run
cd "${GITHUB_WORKSPACE}/testing/0/user_0/"
cargo run
cd "${GITHUB_WORKSPACE}/testing/1 - loading errors/user_1/"
env "RETURN=ok" cargo run
env "RETURN=error" cargo run
env "RETURN=panic" cargo run

cd "${GITHUB_WORKSPACE}/testing/version_compatibility/user_0"
cargo run

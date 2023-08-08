
check-deps:
	cargo --version; yarn --version; node --version; wasm-pack --version;

serve-web:
	cd web; yarn; yarn serve --dev

watch-wasm:
	export CARGO_BUILD_JOBS=3; cd rust; cargo watch -i .gitignore -i "pkg/*" -s "wasm-pack build --debug --target=no-modules"

test:
	export CARGO_BUILD_JOBS=3; cd rust; cargo test --lib

test-main:
	export CARGO_BUILD_JOBS=3; cd rust; cargo test --release --bin main --features profiling

watch-test:
	export CARGO_BUILD_JOBS=3; cd rust; cargo watch -i .gitignore -i "pkg/*" -s "cargo test --lib"

perft:
	cd rust; cargo build --release --bin main --features profiling; cd ..; ./target/release/main perft

uci:
	cd rust; cargo build --release --bin main --features profiling; cd ..; ./target/release/main
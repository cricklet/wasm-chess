
check-deps:
	cargo --version; yarn --version; node --version; wasm-pack --version; cargo watch --version;

# frontend

serve-web:
	cd web; yarn; yarn serve --dev

# wasm

watch-wasm:
	export CARGO_BUILD_JOBS=3; cd rust; cargo watch -i .gitignore -i "pkg/*" -s "wasm-pack build --debug --target=no-modules"

# TODO: --features wasm"
# main

test:
	export CARGO_BUILD_JOBS=3; cd rust; cargo test --release --bin main --features profiling

perft:
	cd rust; cargo build --release --bin main --features profiling; cd ..; ./target/release/main perft

uci:
	cd rust; cargo build --release --bin main --features profiling; cd ..; ./target/release/main

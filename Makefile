
check-deps:
	cargo --version; yarn --version; node --version; wasm-pack --version; cargo watch --version;

# frontend

serve:
	cd web; yarn; yarn serve --dev

# wasm

wasm:
	export CARGO_BUILD_JOBS=3; cd wasm-chess; cargo watch -i .gitignore -i "pkg/*" -s "wasm-pack build --debug --target=no-modules"

# main

test:
	export CARGO_BUILD_JOBS=3; cd rust-chess; cargo test --release --bin main

perft:
	cd rust-chess; cargo run --release --bin perft --features profiling

uci:
	cd rust-chess; cargo run --release --bin main

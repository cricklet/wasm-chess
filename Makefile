
check-deps:
	cargo --version; yarn --version; node --version; wasm-pack --version;

serve-web:
	cd web; yarn; yarn serve --dev

watch-wasm:
	export CARGO_BUILD_JOBS=3; cd rust; cargo watch -i .gitignore -i "pkg/*" -s "wasm-pack build --debug --target=no-modules"

test:
	export CARGO_BUILD_JOBS=3; cd rust; cargo test

watch-test:
	export CARGO_BUILD_JOBS=3; cd rust; cargo watch -i .gitignore -i "pkg/*" -s "cargo test --bin wasm-chess"

perft:
	cd rust; cargo build --release; cd ..; ./target/release/wasm-chess perft
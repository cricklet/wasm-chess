
check-deps:
	cargo --version; yarn --version; node --version;

serve-web:
	cd web; yarn; yarn serve --dev

watch-wasm:
	export CARGO_BUILD_JOBS=3; cd rust; cargo install wasm-pack; cargo watch -i .gitignore -i "pkg/*" -s "wasm-pack build --debug --target=no-modules"

perft:
	cd rust; cargo test --release test_perft_start_board -- --nocapture --test-threads=1 
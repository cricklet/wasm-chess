
check-deps:
	cargo --version; yarn --version; node --version; wasm-pack --version;

serve-web:
	cd web; yarn; yarn serve --dev

watch-wasm:
	export CARGO_BUILD_JOBS=3; cd rust; cargo watch -i .gitignore -i "pkg/*" -s "wasm-pack build --debug --target=no-modules"

watch-main-tests:
	export CARGO_BUILD_JOBS=3; cd rust; cargo watch -i .gitignore -i "pkg/*" -s "cargo test --bin crab-chess"

watch-lib-tests:
	export CARGO_BUILD_JOBS=3; cd rust; cargo watch -i .gitignore -i "pkg/*" -s "cargo test --lib"

perft:
	cd rust; cargo test --bin crab-chess --release test_perft_start_board -- --nocapture --test-threads=1

profile-perft:
	cd rust; cargo test --bin crab-chess --release test_profile_perft_start_board -- --nocapture --test-threads=1

serve-web:
	cd web; yarn serve --dev

watch-wasm:
	cd rust; cargo watch -i .gitignore -i "pkg/*" -s "CARGO_BUILD_JOBS=3 cargo run wasm-pack build --debug --target=no-modules"
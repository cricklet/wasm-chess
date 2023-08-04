
serve:
	cd web; yarn serve

build:
	cd rust; cargo watch -i .gitignore -i "pkg/*" -s "wasm-pack build --target=no-modules"
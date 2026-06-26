doc:
	cargo doc --no-deps -p lrc_rs --all-features

open-doc:
	cargo doc --no-deps -p lrc_rs --all-features --open

test:
	cargo test
	cargo test --no-default-features

check:
	cargo fmt --check
	just test
	just doc
	cargo deny check

loc:
	cloc src/

release:
	cargo build --release

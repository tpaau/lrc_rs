doc:
	RUSTDOCFLAGS="--cfg docsrs" cargo +nightly doc --all-features --no-deps -p lrc_rs

open-doc:
	RUSTDOCFLAGS="--cfg docsrs" cargo +nightly doc --all-features --no-deps -p lrc_rs --open

test:
	cargo test --all-features
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

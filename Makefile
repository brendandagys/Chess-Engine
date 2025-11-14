.PHONY: test

# 32 MB (2^25) stack size
test:
	RUST_MIN_STACK=33554432 cargo test --release

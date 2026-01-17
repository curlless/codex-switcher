.PHONY: fmt clippy test check precommit

fmt:
	cargo fmt --all

clippy:
	cargo clippy --all-targets -- -D warnings

test:
	cargo test --tests

check: fmt clippy test

precommit: check

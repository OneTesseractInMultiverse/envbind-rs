CARGO ?= cargo

.PHONY: all build check clippy doc fmt fmt-check help lint package package-list publish-dry-run test test-doc verify

all: verify

help:
	@printf '%s\n' \
		'build           Build the crate' \
		'check           Run cargo check for all targets' \
		'clippy          Run Clippy with warnings denied' \
		'doc             Build docs.rs-style documentation without dependencies' \
		'fmt             Format all Rust code' \
		'fmt-check       Check formatting' \
		'lint            Alias for clippy' \
		'package         Validate crate package contents' \
		'package-list    List files included in the crate package' \
		'publish-dry-run Run cargo publish dry-run' \
		'test            Run all target tests' \
		'test-doc        Run rustdoc examples' \
		'verify          Run the local quality gate'

build:
	$(CARGO) build

check:
	$(CARGO) check --all-targets

clippy:
	$(CARGO) clippy --all-targets --all-features -- -D warnings

lint: clippy

doc:
	RUSTDOCFLAGS="--cfg docsrs -D warnings" $(CARGO) doc --no-deps --all-features

fmt:
	$(CARGO) fmt --all

fmt-check:
	$(CARGO) fmt --all -- --check

package:
	$(CARGO) package

package-list:
	$(CARGO) package --list

publish-dry-run: verify
	$(CARGO) publish --dry-run

test:
	$(CARGO) test --all-targets

test-doc:
	$(CARGO) test --doc

verify: fmt-check check lint test test-doc doc

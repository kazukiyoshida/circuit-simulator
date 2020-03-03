SHELL=/bin/bash
CARGO = cargo

# Build the project in debug mode
.PHONY: build
build:
	$(CARGO) build $(CARGO_FLAGS)

# Check the format of the source code
.PHONY: fmt-check
fmt:
	cargo fmt --all -- --check

# fmt fix
.PHONY: fmt
fmt:
	cargo fmt

# Run the tests
.PHONY: test
test:
	$(CARGO) test -- --nocapture

.PHONY: sample
sample:
	$(CARGO) run --example sample;

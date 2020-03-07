SHELL=/bin/bash
CARGO = cargo
WASMPACK = wasm-pack

# Build the project in debug mode
.PHONY: build
build:
	$(CARGO) build $(CARGO_FLAGS)

# Wasm build
.PHONY: wasm
wasm:
	$(WASMPACK) build --scope kazukiyoshida

# Publis wasm package
.PHONY: npm-publish
npm-publish:
	cd pkg; npm publish --registry http://localhost:4873

# Check the format of the source code
.PHONY: fmt-check
fmt:
	$(CARGO) fmt --all -- --check

# fmt fix
.PHONY: fmt
fmt:
	$(CARGO) fmt

# Run the tests
.PHONY: test
test:
	$(CARGO) test -- --nocapture

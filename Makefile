.PHONY: all build release test clean fmt lint check run examples doc install help

# Project variables
BINARY_NAME := schema-tui
CARGO := cargo
INSTALL_PATH := ~/.cargo/bin

# Default target
all: build

# Build debug version
build:
	$(CARGO) build

# Build release version
release:
	$(CARGO) build --release

# Run tests
test:
	$(CARGO) test

# Run tests with output
test-verbose:
	$(CARGO) test -- --nocapture

# Clean build artifacts
clean:
	$(CARGO) clean

# Format code
fmt:
	$(CARGO) fmt

# Check formatting
fmt-check:
	$(CARGO) fmt --check

# Run clippy linter
lint:
	$(CARGO) clippy -- -D warnings

# Run clippy with all features
lint-all:
	$(CARGO) clippy --all-features -- -D warnings

# Check without building
check:
	$(CARGO) check

# Check all targets
check-all:
	$(CARGO) check --all-targets

# Run example: dynamic_options
example-dynamic:
	$(CARGO) run --example dynamic_options

# Run example: simple_config
example-simple:
	$(CARGO) run --example simple_config

# Run all examples
examples: example-dynamic example-simple

# Generate documentation
doc:
	$(CARGO) doc --no-deps --open

# Generate documentation without opening
doc-build:
	$(CARGO) doc --no-deps

# Install locally
install:
	$(CARGO) install --path .

# Uninstall
uninstall:
	$(CARGO) uninstall $(BINARY_NAME)

# Watch mode (requires cargo-watch)
watch:
	$(CARGO) watch -x build

# Watch and test (requires cargo-watch)
watch-test:
	$(CARGO) watch -x test

# Benchmark (if benchmarks exist)
bench:
	$(CARGO) bench

# Update dependencies
update:
	$(CARGO) update

# Audit dependencies (requires cargo-audit)
audit:
	$(CARGO) audit

# Full CI check (format, lint, test, build)
ci: fmt-check lint test build

# Help target
help:
	@echo "Available targets:"
	@echo "  build          - Build debug version"
	@echo "  release        - Build optimized release version"
	@echo "  test           - Run tests"
	@echo "  test-verbose   - Run tests with output"
	@echo "  clean          - Remove build artifacts"
	@echo "  fmt            - Format code"
	@echo "  fmt-check      - Check code formatting"
	@echo "  lint           - Run clippy linter"
	@echo "  lint-all       - Run clippy with all features"
	@echo "  check          - Check code without building"
	@echo "  check-all      - Check all targets"
	@echo "  examples       - Run all examples"
	@echo "  example-dynamic - Run dynamic_options example"
	@echo "  example-simple  - Run simple_config example"
	@echo "  doc            - Generate and open documentation"
	@echo "  doc-build      - Generate documentation"
	@echo "  install        - Install locally"
	@echo "  uninstall      - Uninstall"
	@echo "  watch          - Watch and rebuild on changes"
	@echo "  watch-test     - Watch and test on changes"
	@echo "  update         - Update dependencies"
	@echo "  audit          - Audit dependencies"
	@echo "  ci             - Run all CI checks"
	@echo "  help           - Show this help message"

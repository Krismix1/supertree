# List available recipes
default:
    @just --list

# Build the project
build:
    cargo build

# Run tests with coverage and generate HTML report
coverage:
    cargo llvm-cov --html

# Run tests with coverage and generate lcov report
coverage-lcov:
    cargo llvm-cov --lcov --output-path lcov.info

# Run all tests
test:
    cargo test

# Run against a specific target directory
run TARGET_DIR:
    cargo run -- {{TARGET_DIR}}

# Clean build artifacts
clean:
    cargo clean

# Format code
fmt:
    cargo fmt

# Check code formatting
fmt-check:
    cargo fmt --check

# Run clippy lints
lint:
    cargo clippy -- -D warnings 
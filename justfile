# hypercube-utils - TUI utilities for Hypercube Linux

# Default recipe: show available commands
default:
    @just --list

# Build debug version
build:
    cargo build

# Build release version
release:
    cargo build --release

# Run greeter in dryrun mode
greeter *ARGS: build
    ./target/debug/hypercube-greeter --dryrun {{ARGS}}

# Run greeter release build in dryrun mode
greeter-release *ARGS: release
    ./target/release/hypercube-greeter --dryrun {{ARGS}}

# Run onboard wizard in demo mode
onboard *ARGS: build
    ./target/debug/hypercube-onboard --dryrun {{ARGS}}

# Run onboard wizard in release mode
onboard-release *ARGS: release
    ./target/release/hypercube-onboard --dryrun {{ARGS}}

# Run tests
test:
    cargo test

# Run clippy lints
lint:
    cargo clippy -- -W clippy::pedantic -A clippy::must_use_candidate

# Format code
fmt:
    cargo fmt

# Check formatting without modifying
fmt-check:
    cargo fmt -- --check

# Check compilation without building
check:
    cargo check

# Clean build artifacts
clean:
    cargo clean

# Run all checks (format, lint, test, build)
ci: fmt-check lint test release

# Install to system (requires root)
install: release
    sudo install -Dm755 target/release/hypercube-greeter /usr/local/bin/hypercube-greeter
    sudo install -Dm755 target/release/hypercube-onboard /usr/local/bin/hypercube-onboard

# Uninstall from system
uninstall:
    sudo rm -f /usr/local/bin/hypercube-greeter
    sudo rm -f /usr/local/bin/hypercube-onboard

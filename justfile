# vimgreet - neovim-inspired greetd greeter

# Default recipe: show available commands
default:
    @just --list

# Build debug version
build:
    cargo build

# Build release version
release:
    cargo build --release

# Run in dryrun mode (for testing without greetd)
# Usage: just demo [ARGS]
# Examples:
#   just demo
#   just demo --onboard
demo *ARGS: build
    ./target/debug/vimgreet --dryrun {{ARGS}}

# Run release build in dryrun mode
# Usage: just demo-release [ARGS]
demo-release *ARGS: release
    ./target/release/vimgreet --dryrun {{ARGS}}

# Run with debug logging to file
# Usage: just demo-debug [ARGS]
demo-debug *ARGS: build
    RUST_LOG=debug ./target/debug/vimgreet --dryrun --log-file vimgreet.log {{ARGS}}

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

# Watch for changes and rebuild (requires cargo-watch)
watch:
    cargo watch -x check

# Install to system (requires root)
install: release
    sudo install -Dm755 target/release/vimgreet /usr/local/bin/vimgreet

# Uninstall from system
uninstall:
    sudo rm -f /usr/local/bin/vimgreet

# Show binary size
size: release
    ls -lh target/release/vimgreet
    @echo "Stripped size:"
    @strip -s target/release/vimgreet -o /tmp/vimgreet-stripped && ls -lh /tmp/vimgreet-stripped

# Run all checks (format, lint, test, build)
ci: fmt-check lint test release

# Generate documentation
doc:
    cargo doc --no-deps --open

# Update dependencies
update:
    cargo update

# Show dependency tree
deps:
    cargo tree

# Run with logging to file
# Usage: just demo-log [ARGS]
demo-log *ARGS: build
    RUST_LOG=info ./target/debug/vimgreet --dryrun --log-file vimgreet.log {{ARGS}}

# Run onboard wizard in demo mode
onboard: build
    ./target/debug/vimgreet --onboard --config examples/demo.toml

# Run onboard wizard in release mode
onboard-release: release
    ./target/release/vimgreet --onboard --config examples/demo.toml

# Run onboard wizard with debug logging
onboard-debug: build
    RUST_LOG=debug ./target/debug/vimgreet --onboard --config examples/demo.toml --log-file onboard.log

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

# Run in demo mode (for testing without greetd)
demo: build
    ./target/debug/vimgreet --demo

# Run release build in demo mode
demo-release: release
    ./target/release/vimgreet --demo

# Run with debug logging to file
demo-debug: build
    RUST_LOG=debug ./target/debug/vimgreet --demo --log-file vimgreet.log

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
demo-log: build
    RUST_LOG=info ./target/debug/vimgreet --demo --log-file vimgreet.log

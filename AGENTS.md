# AGENTS.md

This file provides guidance to coding agents when working with code in this repository.

## Project Overview

hypercube-utils: TUI utilities for Hypercube Linux built with Rust (edition 2024), using ratatui + crossterm for terminal UI and tokio for async operations. Produces two binaries:

- **hypercube-greeter** - Vim-inspired TUI login greeter for the greetd login manager
- **hypercube-onboard** - First-boot onboarding wizard for initial system setup

## Build Commands

Uses `just` as the task runner:

```
just build          # Debug build
just release        # Optimized release build
just test           # Run tests
just lint           # Clippy with pedantic checks
just fmt            # Format code
just fmt-check      # Check formatting without changes
just check          # Type check only
just ci             # Full pipeline: fmt-check → lint → test → release
just greeter        # Run greeter in dryrun mode
just onboard        # Run onboard with examples/demo.toml in dryrun mode
```

Run a single test: `cargo test <test_name>`

## Architecture

### Shared Library (`src/lib.rs`)

Both binaries share common infrastructure through the library crate:

- **`vim/`** - Modal editing system (Normal/Insert/Command modes) with `VimMode`, `InputBuffer`, and command parsing
- **`event/`** - Async event handler for keyboard/mouse/tick events (250ms tick rate)
- **`ipc/`** - `GreetdClient` for greetd IPC authentication protocol
- **`system/`** - User discovery, session discovery (Wayland/X11 desktop files), power management
- **`ui/`** - Color theming
- **`error.rs`** - `HypercubeError` enum via thiserror

### Greeter (`src/greeter/`, binary: `src/bin/greeter.rs`)

Login greeter communicating with greetd. State machine with vim modal input for username/password entry. Supports `:user`/`:session` pickers, `:reboot`/`:poweroff` commands.

### Onboard (`src/onboard/`, binary: `src/bin/onboard.rs`)

Multi-step wizard driven by TOML configuration (`examples/demo.toml`). Steps include user creation, locale, keyboard, timezone, network, package installation, and a review step that applies all settings. Uses an executor (`executor.rs`) to run system setup commands. Configuration structures in `config.rs`.

- **Non-blocking execution**: System commands (user creation, locale/keymap/timezone, package install) run in background tasks via `tokio::spawn` + `spawn_blocking`. The main loop uses `tokio::select!` to concurrently process input events and `ExecutionMessage` channel updates. Methods `start_review_execution()`, `start_update_execution()`, and `start_step_execution()` return `Option<UnboundedReceiver<ExecutionMessage>>`. In dryrun mode they return `None` and use tick-based simulation instead.
- **Picker pages** (Locale, Keyboard, Preferences): Auto-enter insert mode on focus so the filter is immediately editable. Typing printable characters in normal mode also auto-activates the filter.
- **Update step**: Per-category independent scrolling via `update_category_scroll: Vec<usize>`. Each category gets an equal vertical slice with its own scroll offset, adjusted at render time to keep the cursor visible. `ui::draw` takes `&mut OnboardApp` to support this.
- **Deferred execution**: Picker selections only store the chosen value; actual system commands are deferred to the Review step.

### Key Design Patterns

- Both apps use `--dryrun` flag for testing without real system changes (simulates commands and user discovery)
- Passwords handled with `zeroize` for secure memory clearing
- Terminal state (raw mode, alternate screen) properly restored on exit/panic
- Async event loop with tokio; non-blocking IPC and command execution
- Onboard execution uses channel-based message passing (`ExecutionMessage` enum) to decouple background work from UI updates

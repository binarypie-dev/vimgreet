# hypercube-utils

TUI utilities for Hypercube Linux.

## Binaries

### hypercube-greeter

A vim-inspired TUI greeter for [greetd](https://sr.ht/~kennylevinsen/greetd/).

- **Full vim modal editing** - Normal, Insert, and Command modes
- **Vim keybindings** - `hjkl` navigation, `i`/`a` to insert, `Escape` to exit
- **Command mode** - `:reboot`, `:poweroff`, `:session`, `:user`, `:help`
- **Session discovery** - Automatically finds Wayland and X11 sessions
- **User discovery** - Lists available users from `/etc/passwd`
- **Demo mode** - Test the UI without greetd using `--dryrun`

### hypercube-onboard

A first-boot onboarding wizard for system setup.

- **User creation** - Create user account with password
- **Locale/keyboard/timezone** - System configuration
- **Network** - WiFi setup via external tool
- **Package installation** - Flatpaks, Homebrew, Distrobox containers
- **Vim modal editing** - Same vim keybindings as the greeter

## Installation

### From COPR (Fedora)

```bash
sudo dnf copr enable binarypie/hypercube
sudo dnf install hypercube-utils
```

### From source

```bash
cargo build --release
sudo install -Dm755 target/release/hypercube-greeter /usr/local/bin/hypercube-greeter
sudo install -Dm755 target/release/hypercube-onboard /usr/local/bin/hypercube-onboard
```

## Configuration

### Greeter

hypercube-greeter is a TUI application that needs to run inside a terminal emulator.
Configure greetd to launch it within a compositor like cage:

#### /etc/greetd/config.toml

```toml
[terminal]
vt = 1

[default_session]
command = "cage -s -- foot -e hypercube-greeter"
user = "greeter"
```

Alternative with alacritty:

```toml
[default_session]
command = "cage -s -- alacritty -e hypercube-greeter"
user = "greeter"
```

Alternative with ghostty:

```toml
[default_session]
command = "cage -s -- ghostty -e hypercube-greeter"
user = "greeter"
```

For multi-monitor setups with wallpaper, use sway instead of cage:

```toml
[default_session]
command = "sway --config /etc/greetd/sway-config"
user = "greeter"
```

With `/etc/greetd/sway-config`:

```
output * bg /path/to/wallpaper.png fill
exec "foot -e hypercube-greeter; swaymsg exit"
```

### Onboard

hypercube-onboard reads its configuration from `/etc/hypercube/onboard.toml`. See `examples/onboard.toml` for the full config format.

```bash
hypercube-onboard --config /path/to/config.toml
```

## Keybindings

### Normal Mode

| Key | Action |
|-----|--------|
| `h` / `l` | Move cursor left/right |
| `j` / `k` | Move between fields |
| `i` | Enter insert mode |
| `a` | Enter insert mode (after cursor) |
| `:` | Enter command mode |
| `x` | Delete character |
| `dd` | Clear field |
| `Enter` | Login |
| `F2` | Open user picker |
| `F3` | Open session picker |
| `F12` | Power menu |

### Insert Mode

| Key | Action |
|-----|--------|
| `Escape` | Return to normal mode |
| `Enter` | Submit / next field |
| `Ctrl+u` | Clear line |
| `Ctrl+w` | Delete word |

### Commands

| Command | Action |
|---------|--------|
| `:session [name]` | Select session |
| `:user [name]` | Select user |
| `:reboot` | Reboot system |
| `:poweroff` | Shutdown system |
| `:help` | Show help |
| `:q` | Login |

## Development

```bash
# Run greeter in dryrun mode
just greeter

# Run onboard in dryrun mode
just onboard

# Run onboard with a config file
just onboard --config examples/demo.toml

# Build release
just release

# Run all checks
just ci
```

## License

Apache-2.0

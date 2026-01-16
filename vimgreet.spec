%global crate vimgreet

Name:           %{crate}
Version:        0.1.0
Release:        1%{?dist}
Summary:        A neovim-inspired greeter for greetd

License:        Apache-2.0
URL:            https://github.com/binarypie/vimgreet
Source0:        %{url}/archive/v%{version}/%{crate}-%{version}.tar.gz

BuildRequires:  rust >= 1.70
BuildRequires:  cargo
BuildRequires:  gcc

Requires:       greetd

%description
A neovim-inspired TUI greeter for greetd with full vim modal editing support.
Features include normal/insert/command modes, hjkl navigation, and vim-style
commands like :reboot, :poweroff, :session, and :help.

Designed to run inside a terminal emulator within a Wayland compositor like
cage or sway.

%prep
%autosetup -n %{crate}-%{version}

%build
cargo build --release --locked

%install
install -Dm755 target/release/%{crate} %{buildroot}%{_bindir}/%{crate}

# Install example greetd config
install -Dm644 /dev/stdin %{buildroot}%{_docdir}/%{crate}/greetd-config.toml << 'EOF'
[terminal]
vt = 1

[default_session]
command = "cage -s -- foot -e vimgreet"
user = "greeter"
EOF

%files
%license LICENSE
%doc README.md
%{_bindir}/%{crate}
%{_docdir}/%{crate}/greetd-config.toml

%changelog
* Wed Jan 15 2025 binarypie <binarypie@users.noreply.github.com> - 0.1.0-1
- Initial package

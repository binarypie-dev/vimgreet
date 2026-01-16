%global crate vimgreet

Name:           %{crate}
Version:        0.1.0
Release:        1%{?dist}
Summary:        A neovim-inspired greeter for greetd

License:        Apache-2.0
URL:            https://github.com/binarypie/vimgreet
Source0:        %{url}/archive/main/%{crate}-main.tar.gz

BuildRequires:  rust >= 1.70
BuildRequires:  cargo

Requires:       greetd

%description
A neovim-inspired TUI greeter for greetd with full vim modal editing support.
Features include normal/insert/command modes, hjkl navigation, and vim-style
commands like :reboot, :poweroff, :session, and :help.

Designed to run inside a terminal emulator within a Wayland compositor like
cage or sway.

%prep
%autosetup -n %{crate}-main

%build
cargo build --release --locked

%install
install -Dm755 target/release/%{crate} %{buildroot}%{_bindir}/%{crate}

%files
%license LICENSE
%doc README.md
%{_bindir}/%{crate}

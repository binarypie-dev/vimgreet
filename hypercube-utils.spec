%global crate hypercube-utils

Name:           %{crate}
Version:        0.1.4
Release:        1%{?dist}
Summary:        TUI utilities for Hypercube Linux

License:        Apache-2.0
URL:            https://github.com/binarypie-dev/hypercube-utils
Source0:        %{url}/archive/main/%{crate}-main.tar.gz

BuildRequires:  rust >= 1.70
BuildRequires:  cargo

Requires:       greetd

%description
TUI utilities for Hypercube Linux including a vim-inspired greeter for greetd
and a first-boot onboarding wizard.

%prep
%autosetup -n %{crate}-main

%build
cargo build --release --locked

%install
install -Dm755 target/release/hypercube-greeter %{buildroot}%{_bindir}/hypercube-greeter
install -Dm755 target/release/hypercube-onboard %{buildroot}%{_bindir}/hypercube-onboard

%files
%license LICENSE
%doc README.md
%{_bindir}/hypercube-greeter
%{_bindir}/hypercube-onboard

#!/usr/bin/env bash
# Install HT32 Panel from source on supported x86_64 Linux distributions.
set -Eeuo pipefail

REPO="${HT32_PANEL_REPO:-ayushpandey-ops/ht32-panel}"
BRANCH="${HT32_PANEL_BRANCH:-main}"
BASE_URL="https://github.com/${REPO}/archive/refs/heads/${BRANCH}.tar.gz"

info() { printf '\n\033[1;34m[ht32-panel]\033[0m %s\n' "$*"; }
fail() { printf '\n\033[1;31m[ht32-panel] ERROR:\033[0m %s\n' "$*" >&2; exit 1; }
STEP=0
step() {
  STEP=$((STEP + 1))
  printf '\n\033[1;36m[%d/6]\033[0m %s\n' "$STEP" "$*"
}

[[ "$(uname -s)" == Linux ]] || fail "This installer supports Linux only."
[[ "$(uname -m)" == x86_64 || "$(uname -m)" == amd64 ]] || fail "This release currently supports x86_64 systems only."
command -v sudo >/dev/null 2>&1 || fail "sudo is required to install system dependencies and configure USB access."

if [[ "${EUID}" -eq 0 ]]; then
  fail "Run this installer as your normal user, not with sudo. It will ask sudo when needed."
fi

TMP_DIR="$(mktemp -d)"
trap 'rm -rf "$TMP_DIR"' EXIT

install_build_dependencies() {
  if command -v apt-get >/dev/null 2>&1; then
    info "Installing build and USB dependencies with apt."
    sudo apt-get update
    sudo apt-get install -y build-essential pkg-config libusb-1.0-0-dev libudev-dev curl ca-certificates tar gzip
  elif command -v dnf >/dev/null 2>&1; then
    info "Installing build and USB dependencies with dnf."
    sudo dnf install -y gcc gcc-c++ make pkgconf-pkg-config libusb1-devel systemd-devel curl ca-certificates tar gzip
  elif command -v pacman >/dev/null 2>&1; then
    info "Installing build and USB dependencies with pacman."
    sudo pacman -Sy --needed --noconfirm base-devel pkgconf libusb systemd curl ca-certificates tar gzip
  else
    fail "Unsupported package manager. Supported systems use apt, dnf, or pacman."
  fi
}

step "Install system build and USB dependencies."
install_build_dependencies

step "Set up the Rust stable toolchain."
if ! command -v rustup >/dev/null 2>&1; then
  info "Installing the Rust stable toolchain for your user."
  curl --proto '=https' --tlsv1.2 -fsSL \
    https://static.rust-lang.org/rustup/dist/x86_64-unknown-linux-gnu/rustup-init \
    -o "$TMP_DIR/rustup-init"
  chmod 0755 "$TMP_DIR/rustup-init"
  "$TMP_DIR/rustup-init" -y --profile minimal
fi

export PATH="${HOME}/.cargo/bin:${PATH}"
command -v cargo >/dev/null 2>&1 || fail "Rust installation finished, but cargo is not available in PATH."
rustup toolchain install stable --profile minimal
rustup default stable

step "Download source from ${REPO} (${BRANCH})."
curl --proto '=https' --tlsv1.2 -fsSL "$BASE_URL" -o "$TMP_DIR/source.tar.gz"
mkdir "$TMP_DIR/source"
tar -xzf "$TMP_DIR/source.tar.gz" --strip-components=1 -C "$TMP_DIR/source"

step "Build the daemon and command-line client. This may take a few minutes."
cargo build --release --manifest-path "$TMP_DIR/source/Cargo.toml" \
  -p ht32-panel-daemon -p ht32-panel-cli

step "Install binaries, configuration, D-Bus policy, and USB rules."
if ! getent group plugdev >/dev/null 2>&1; then
  sudo groupadd --system plugdev
fi
sudo install -Dm755 "$TMP_DIR/source/target/release/ht32paneld" /usr/bin/ht32paneld
sudo install -Dm755 "$TMP_DIR/source/target/release/ht32panelctl" /usr/bin/ht32panelctl
sudo install -Dm644 "$TMP_DIR/source/packaging/ht32-panel.service" /usr/lib/systemd/system/ht32-panel.service
sudo install -Dm644 "$TMP_DIR/source/packaging/org.ht32panel.Daemon.conf" /usr/share/dbus-1/system.d/org.ht32panel.Daemon.conf
sudo install -Dm644 "$TMP_DIR/source/packaging/99-ht32-panel.rules" /etc/udev/rules.d/99-ht32-panel.rules
sudo install -d -m 0750 /etc/ht32-panel
if [[ ! -e /etc/ht32-panel/config.toml ]]; then
  sudo install -m 0644 "$TMP_DIR/source/config/default.toml" /etc/ht32-panel/config.toml
else
  info "Keeping existing /etc/ht32-panel/config.toml."
fi

# Retire the per-user service if this machine was previously set up that way;
# running both instances can make them contend for the same USB device.
if command -v systemctl >/dev/null 2>&1; then
  systemctl --user disable --now ht32-panel.service >/dev/null 2>&1 || true
fi

step "Reload system services and start HT32 Panel."
sudo systemctl daemon-reload
sudo udevadm control --reload-rules
sudo udevadm trigger
sudo systemctl reload dbus >/dev/null 2>&1 || sudo systemctl reload dbus-broker >/dev/null 2>&1 || true
sudo systemctl enable --now ht32-panel.service

info "Installation complete. Service status:"
sudo systemctl --no-pager --full status ht32-panel.service || true
printf '\nWeb UI: http://[::1]:8686\n'
printf 'Logs:   sudo journalctl -u ht32-panel.service -f\n'
printf 'CLI:    ht32panelctl --help\n'

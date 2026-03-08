#!/bin/sh
# install.sh - pycu installer
#
# Usage:
#   curl -Ls https://raw.githubusercontent.com/Logic-py/python-check-updates/main/install.sh | sh
#
# Override the install directory:
#   PYCU_INSTALL_DIR=/usr/local/bin curl -Ls ... | sh

set -e

REPO="Logic-py/python-check-updates"
BINARY="pycu"
INSTALL_DIR="${PYCU_INSTALL_DIR:-${HOME}/.local/bin}"

# -- Helpers ------------------------------------------------------------------
say()  { printf "\033[1m%s\033[0m\n" "$*"; }
info() { printf "  \033[36m•\033[0m %s\n" "$*"; }
err()  { printf "\033[31merror:\033[0m %s\n" "$*" >&2; exit 1; }

if command -v curl > /dev/null 2>&1; then
  fetch() { curl -sL "$1"; }
  fetch_to() { curl -sL "$1" -o "$2"; }
elif command -v wget > /dev/null 2>&1; then
  fetch() { wget -qO- "$1"; }
  fetch_to() { wget -qO "$2" "$1"; }
else
  err "curl or wget is required"
fi

# -- Detect OS -----------------------------------------------------------------
OS=$(uname -s 2>/dev/null || echo "unknown")
ARCH=$(uname -m 2>/dev/null || echo "unknown")

case "$OS" in
  Linux)                   os="linux"   ;;
  Darwin)                  os="darwin"  ;;
  MINGW*|CYGWIN*|MSYS*)   os="windows" ;;
  *) err "unsupported operating system: $OS" ;;
esac

case "$ARCH" in
  x86_64|amd64)    arch="x86_64"  ;;
  aarch64|arm64)   arch="aarch64" ;;
  *) err "unsupported architecture: $ARCH" ;;
esac

# -- Map to the Rust target triple used in the release -------------------------
case "${os}-${arch}" in
  linux-x86_64)    target="x86_64-unknown-linux-musl"  ; ext="tar.gz" ;;
  linux-aarch64)   target="aarch64-unknown-linux-musl" ; ext="tar.gz" ;;
  darwin-x86_64)   target="x86_64-apple-darwin"        ; ext="tar.gz" ;;
  darwin-aarch64)  target="aarch64-apple-darwin"       ; ext="tar.gz" ;;
  windows-x86_64)  target="x86_64-pc-windows-msvc"     ; ext="zip"    ;;
  *)
    err "no pre-built binary for ${os}-${arch}. Build from source: https://github.com/${REPO}"
    ;;
esac

# -- Resolve latest version ----------------------------------------------------
say "Fetching latest ${BINARY} release..."

VERSION=$(fetch "https://api.github.com/repos/${REPO}/releases/latest" \
  | grep '"tag_name"' \
  | sed 's/.*"tag_name" *: *"\([^"]*\)".*/\1/')

[ -n "$VERSION" ] || err "could not determine latest version - is the repository public?"

info "version : ${VERSION}"
info "platform: ${os}-${arch} (${target})"
info "install : ${INSTALL_DIR}"

# -- Download ------------------------------------------------------------------
ARCHIVE="${BINARY}-${target}.${ext}"
URL="https://github.com/${REPO}/releases/download/${VERSION}/${ARCHIVE}"

TMP_DIR=$(mktemp -d)
trap 'rm -rf "$TMP_DIR"' EXIT INT TERM

say "Downloading ${ARCHIVE}..."
fetch_to "$URL" "${TMP_DIR}/${ARCHIVE}" || err "download failed: ${URL}"

# -- Extract -------------------------------------------------------------------
case "$ext" in
  tar.gz)
    tar -xzf "${TMP_DIR}/${ARCHIVE}" -C "$TMP_DIR"
    BIN="${TMP_DIR}/${BINARY}"
    ;;
  zip)
    command -v unzip > /dev/null 2>&1 || err "unzip is required"
    unzip -q "${TMP_DIR}/${ARCHIVE}" -d "$TMP_DIR"
    BIN="${TMP_DIR}/${BINARY}.exe"
    ;;
esac

[ -f "$BIN" ] || err "binary not found in archive - please file a bug at https://github.com/${REPO}/issues"

# -- Install -------------------------------------------------------------------
mkdir -p "$INSTALL_DIR"

DEST="${INSTALL_DIR}/${BINARY}"
[ "$ext" = "zip" ] && DEST="${DEST}.exe"

mv "$BIN" "$DEST"
chmod +x "$DEST"

say "${BINARY} ${VERSION} installed to ${DEST}"

# -- PATH hint -----------------------------------------------------------------
case ":${PATH}:" in
  *":${INSTALL_DIR}:"*) ;;
  *)
    printf "\n\033[33mhint:\033[0m %s is not in your PATH.\n" "$INSTALL_DIR"
    printf "     Add this to your shell profile (~/.bashrc, ~/.zshrc, etc.):\n\n"
    printf "       \033[2mexport PATH=\"%s:\$PATH\"\033[0m\n\n" "$INSTALL_DIR"
    ;;
esac

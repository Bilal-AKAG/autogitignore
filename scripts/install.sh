#!/usr/bin/env sh
set -eu

REPO="Bilal-AKAG/autogitignore"
BIN_NAME="autogitignore"

if [ -n "${1:-}" ]; then
  VERSION="$1"
else
  VERSION="$(curl -fsSL "https://api.github.com/repos/$REPO/releases/latest" | sed -n 's/.*"tag_name": "\([^"]*\)".*/\1/p' | head -n 1)"
fi

if [ -z "$VERSION" ]; then
  echo "Could not determine release version." >&2
  exit 1
fi

OS="$(uname -s)"
ARCH="$(uname -m)"

case "$OS" in
  Linux) OS_PART="unknown-linux-gnu" ;;
  Darwin) OS_PART="apple-darwin" ;;
  *)
    echo "Unsupported OS: $OS" >&2
    exit 1
    ;;
esac

case "$ARCH" in
  x86_64|amd64) ARCH_PART="x86_64" ;;
  arm64|aarch64) ARCH_PART="aarch64" ;;
  *)
    echo "Unsupported architecture: $ARCH" >&2
    exit 1
    ;;
esac

TARGET="$ARCH_PART-$OS_PART"
ARCHIVE="$BIN_NAME-$VERSION-$TARGET.tar.gz"
URL="https://github.com/$REPO/releases/download/$VERSION/$ARCHIVE"

TMP_DIR="$(mktemp -d)"
trap 'rm -rf "$TMP_DIR"' EXIT INT TERM

curl -fL "$URL" -o "$TMP_DIR/$ARCHIVE"
tar -xzf "$TMP_DIR/$ARCHIVE" -C "$TMP_DIR"

INSTALL_DIR="${BINDIR:-$HOME/.local/bin}"
mkdir -p "$INSTALL_DIR"
install "$TMP_DIR/$BIN_NAME" "$INSTALL_DIR/$BIN_NAME"

if [ -t 1 ]; then
  C_RESET='\033[0m'
  C_BOLD='\033[1m'
  C_CYAN='\033[36m'
  C_GREEN='\033[32m'
  C_YELLOW='\033[33m'
else
  C_RESET=''
  C_BOLD=''
  C_CYAN=''
  C_GREEN=''
  C_YELLOW=''
fi

printf '\n%s========================================%s\n' "$C_CYAN" "$C_RESET"
printf '%s%s  Thanks for installing autogitignore%s\n' "$C_BOLD" "$C_GREEN" "$C_RESET"
printf '%s========================================%s\n' "$C_CYAN" "$C_RESET"
printf '\n%sInstalled version:%s %s\n' "$C_BOLD" "$C_RESET" "$VERSION"
printf '%sBinary path:%s %s\n' "$C_BOLD" "$C_RESET" "$INSTALL_DIR/$BIN_NAME"
printf '\n%sTry it now:%s\n' "$C_BOLD" "$C_RESET"
printf '  %sautogitignore%s\n' "$C_GREEN" "$C_RESET"
printf '\n%sIf command is not found, add this to PATH:%s\n' "$C_YELLOW" "$C_RESET"
printf '  export PATH="%s:\$PATH"\n' "$INSTALL_DIR"
printf '\n'

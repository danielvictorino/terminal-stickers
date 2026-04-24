#!/usr/bin/env sh
set -eu

REPO="danielvictorino/terminal-stickers"
BIN_NAME="terminal-stickers"
INSTALL_DIR="${TERMINAL_STICKERS_INSTALL_DIR:-$HOME/.local/bin}"
SHARE_DIR="${TERMINAL_STICKERS_SHARE_DIR:-$INSTALL_DIR/../share/terminal-stickers}"

os="$(uname -s)"
arch="$(uname -m)"

case "$os" in
  Linux) os_part="unknown-linux-gnu" ;;
  Darwin) os_part="apple-darwin" ;;
  *) echo "unsupported OS: $os" >&2; exit 1 ;;
esac

case "$arch" in
  x86_64|amd64) arch_part="x86_64" ;;
  arm64|aarch64)
    if [ "$os" = "Darwin" ]; then
      arch_part="aarch64"
    else
      echo "unsupported Linux architecture: $arch" >&2
      exit 1
    fi
    ;;
  *) echo "unsupported architecture: $arch" >&2; exit 1 ;;
esac

target="${arch_part}-${os_part}"
archive="${BIN_NAME}-${target}.tar.gz"
base_url="https://github.com/${REPO}/releases/latest/download"
tmp_dir="$(mktemp -d)"

cleanup() {
  rm -rf "$tmp_dir"
}
trap cleanup EXIT

download() {
  url="$1"
  output="$2"
  if command -v curl >/dev/null 2>&1; then
    curl -fsSL "$url" -o "$output"
  elif command -v wget >/dev/null 2>&1; then
    wget -q "$url" -O "$output"
  else
    echo "curl or wget is required" >&2
    exit 1
  fi
}

download "${base_url}/${archive}" "${tmp_dir}/${archive}"

if download "${base_url}/${archive}.sha256" "${tmp_dir}/${archive}.sha256" 2>/dev/null; then
  (
    cd "$tmp_dir"
    if command -v sha256sum >/dev/null 2>&1; then
      sha256sum -c "${archive}.sha256"
    elif command -v shasum >/dev/null 2>&1; then
      shasum -a 256 -c "${archive}.sha256"
    else
      echo "checksum file downloaded, but no sha256 tool was found" >&2
    fi
  )
fi

mkdir -p "$INSTALL_DIR"
tar -xzf "${tmp_dir}/${archive}" -C "$tmp_dir"
install -m 0755 "${tmp_dir}/${BIN_NAME}" "${INSTALL_DIR}/${BIN_NAME}"

if [ -d "${tmp_dir}/packs" ]; then
  mkdir -p "${SHARE_DIR}/packs"
  cp -R "${tmp_dir}/packs/." "${SHARE_DIR}/packs/"
fi

echo "installed ${BIN_NAME} to ${INSTALL_DIR}/${BIN_NAME}"
echo "installed sticker packs to ${SHARE_DIR}/packs"
case ":$PATH:" in
  *":$INSTALL_DIR:"*) ;;
  *) echo "add ${INSTALL_DIR} to PATH if ${BIN_NAME} is not found" ;;
esac

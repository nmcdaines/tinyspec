#!/bin/sh
# Install script for tinyspec
# Usage: curl -fsSL https://tinyspec.dev/install.sh | sh
set -eu

REPO="nmcdaines/tinyspec"
INSTALL_DIR="$HOME/.local/bin"

main() {
    check_dependencies
    detect_platform
    fetch_latest_version
    download_and_install
    print_success
}

check_dependencies() {
    if ! command -v curl >/dev/null 2>&1; then
        err "curl is required but not found. Please install curl and try again."
    fi
    if ! command -v tar >/dev/null 2>&1; then
        err "tar is required but not found. Please install tar and try again."
    fi
}

detect_platform() {
    OS="$(uname -s)"
    ARCH="$(uname -m)"

    case "$OS" in
        Darwin)
            case "$ARCH" in
                arm64|aarch64) TARGET="aarch64-apple-darwin" ;;
                x86_64)        TARGET="x86_64-apple-darwin" ;;
                *)             err "Unsupported architecture: $ARCH on macOS" ;;
            esac
            ;;
        Linux)
            case "$ARCH" in
                x86_64|amd64) TARGET="x86_64-unknown-linux-gnu" ;;
                *)            err "Unsupported architecture: $ARCH on Linux" ;;
            esac
            ;;
        *)
            err "Unsupported operating system: $OS. Use 'cargo install tinyspec' instead."
            ;;
    esac

    ARTIFACT="tinyspec-${TARGET}.tar.gz"
    info "Detected platform: ${TARGET}"
}

fetch_latest_version() {
    info "Fetching latest release..."
    VERSION=$(curl -fsSL "https://api.github.com/repos/${REPO}/releases/latest" \
        | grep '"tag_name"' \
        | head -1 \
        | sed 's/.*"tag_name": *"\([^"]*\)".*/\1/')

    if [ -z "$VERSION" ]; then
        err "Failed to determine latest version. Check your internet connection."
    fi

    info "Latest version: ${VERSION}"
    BASE_URL="https://github.com/${REPO}/releases/download/${VERSION}"
}

download_and_install() {
    TMPDIR=$(mktemp -d)
    trap 'rm -rf "$TMPDIR"' EXIT

    info "Downloading ${ARTIFACT}..."
    curl -fsSL "${BASE_URL}/${ARTIFACT}" -o "${TMPDIR}/${ARTIFACT}" \
        || err "Failed to download ${ARTIFACT}"

    info "Downloading checksums..."
    if curl -fsSL "${BASE_URL}/checksums.txt" -o "${TMPDIR}/checksums.txt" 2>/dev/null; then
        verify_checksum
    else
        warn "Checksums not available for this release, skipping verification"
    fi

    info "Extracting..."
    tar xzf "${TMPDIR}/${ARTIFACT}" -C "${TMPDIR}" \
        || err "Failed to extract ${ARTIFACT}"

    mkdir -p "$INSTALL_DIR"
    mv "${TMPDIR}/tinyspec" "${INSTALL_DIR}/tinyspec"
    chmod +x "${INSTALL_DIR}/tinyspec"
}

verify_checksum() {
    EXPECTED=$(grep "${ARTIFACT}" "${TMPDIR}/checksums.txt" | awk '{print $1}')
    if [ -z "$EXPECTED" ]; then
        warn "No checksum found for ${ARTIFACT}, skipping verification"
        return
    fi

    if command -v sha256sum >/dev/null 2>&1; then
        ACTUAL=$(sha256sum "${TMPDIR}/${ARTIFACT}" | awk '{print $1}')
    elif command -v shasum >/dev/null 2>&1; then
        ACTUAL=$(shasum -a 256 "${TMPDIR}/${ARTIFACT}" | awk '{print $1}')
    else
        warn "No SHA256 tool found, skipping checksum verification"
        return
    fi

    if [ "$EXPECTED" != "$ACTUAL" ]; then
        err "Checksum verification failed!
  Expected: ${EXPECTED}
  Actual:   ${ACTUAL}
The downloaded file may be corrupted or tampered with."
    fi

    info "Checksum verified"
}

print_success() {
    echo ""
    echo "  tinyspec ${VERSION} installed to ${INSTALL_DIR}/tinyspec"
    echo ""

    case ":${PATH}:" in
        *":${INSTALL_DIR}:"*) ;;
        *)
            warn "${INSTALL_DIR} is not in your PATH"
            echo ""
            echo "  Add it by appending this to your shell profile (~/.bashrc, ~/.zshrc, etc.):"
            echo ""
            echo "    export PATH=\"${INSTALL_DIR}:\$PATH\""
            echo ""
            ;;
    esac
}

info() {
    echo "  info: $1"
}

warn() {
    echo "  warn: $1" >&2
}

err() {
    echo "  error: $1" >&2
    exit 1
}

main

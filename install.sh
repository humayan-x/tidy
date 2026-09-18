#!/bin/sh
# tidy installer script
# Installs the precompiled binary for Linux and macOS.
#
# Usage:
#   curl -fsSL https://raw.githubusercontent.com/humayan-x/tidy/main/install.sh | sh
#
# Environment variables:
#   TIDY_VERSION : Specify a release version (e.g. 0.1.0). Default: latest release.
#   TIDY_REPO    : GitHub repository (e.g. humayan-x/tidy). Default: humayan-x/tidy.
#   INSTALL_DIR  : Destination directory. Default: ~/.local/bin (or /usr/local/bin if root).

set -e

# Terminal styling
if [ -t 1 ]; then
    RED="\033[1;31m"
    GREEN="\033[1;32m"
    YELLOW="\033[1;33m"
    CYAN="\033[1;36m"
    BOLD="\033[1m"
    RESET="\033[0m"
else
    RED=""
    GREEN=""
    YELLOW=""
    CYAN=""
    BOLD=""
    RESET=""
fi

log_info() {
    printf "  ${CYAN}[tidy]${RESET} %s\n" "$1"
}

log_success() {
    printf "  ${GREEN}✓${RESET} ${BOLD}%s${RESET}\n" "$1"
}

log_warn() {
    printf "  ${YELLOW}⚠${RESET} %s\n" "$1"
}

log_error() {
    printf "  ${RED}✗ Error:${RESET} %s\n" "$1" >&2
}

# 1. Detect Operating System
OS="$(uname -s)"
case "$OS" in
    Linux*)  PLATFORM="linux" ;;
    Darwin*) PLATFORM="darwin" ;;
    *)
        log_error "Unsupported operating system: $OS. tidy supports Linux and macOS."
        exit 1
        ;;
esac

# 2. Detect CPU Architecture
ARCH="$(uname -m)"
case "$ARCH" in
    x86_64|amd64)
        CPU="x86_64"
        ;;
    arm64|aarch64)
        CPU="aarch64"
        ;;
    *)
        log_error "Unsupported CPU architecture: $ARCH. tidy supports x86_64 and aarch64 (Apple Silicon)."
        exit 1
        ;;
esac

# Map to target triple
if [ "$PLATFORM" = "linux" ]; then
    if [ "$CPU" = "x86_64" ]; then
        TARGET="x86_64-unknown-linux-musl"
    else
        log_error "Precompiled Linux binaries are currently provided for x86_64."
        exit 1
    fi
elif [ "$PLATFORM" = "darwin" ]; then
    if [ "$CPU" = "aarch64" ]; then
        TARGET="aarch64-apple-darwin"
    elif [ "$CPU" = "x86_64" ]; then
        TARGET="x86_64-apple-darwin"
    fi
fi

REPO="${TIDY_REPO:-humayan-x/tidy}"

# 3. Determine Version
if [ -z "$TIDY_VERSION" ]; then
    log_info "Detecting latest release from GitHub (${REPO})..."
    LATEST_JSON=$(curl -fsSL "https://api.github.com/repos/${REPO}/releases/latest" 2>/dev/null || true)
    if [ -n "$LATEST_JSON" ]; then
        VERSION=$(printf "%s" "$LATEST_JSON" | grep '"tag_name":' | sed -E 's/.*"([^"]+)".*/\1/' | sed 's/^v//')
    fi
    if [ -z "$VERSION" ]; then
        VERSION="0.2.1"
        log_warn "Could not query GitHub API; falling back to default v${VERSION}."
    fi
else
    VERSION="${TIDY_VERSION#v}"
fi

TAG="v${VERSION}"
TARBALL_NAME="tidy-${TAG}-${TARGET}.tar.gz"
DOWNLOAD_URL="https://github.com/${REPO}/releases/download/${TAG}/${TARBALL_NAME}"
CHECKSUM_URL="https://github.com/${REPO}/releases/download/${TAG}/checksums.txt"

# 4. Determine Destination Directory
if [ -z "$INSTALL_DIR" ]; then
    if [ "$(id -u)" -eq 0 ]; then
        INSTALL_DIR="/usr/local/bin"
    else
        INSTALL_DIR="${HOME}/.local/bin"
    fi
fi

log_info "Preparing to install tidy ${TAG} (${TARGET})..."
log_info "Destination: ${INSTALL_DIR}/tidy"

# 5. Create temporary scratch space
TMP_DIR=$(mktemp -d 2>/dev/null || mktemp -d -t 'tidy-install')
cleanup() {
    rm -rf "$TMP_DIR"
}
trap cleanup EXIT INT TERM

# 6. Download release archive
log_info "Downloading ${TARBALL_NAME}..."
if ! curl -fSL --progress-bar "$DOWNLOAD_URL" -o "${TMP_DIR}/${TARBALL_NAME}"; then
    log_error "Download failed from: $DOWNLOAD_URL"
    exit 1
fi

# 7. Checksum Verification (if checksums.txt is available)
if curl -fsSL "$CHECKSUM_URL" -o "${TMP_DIR}/checksums.txt" 2>/dev/null; then
    log_info "Verifying SHA-256 checksum..."
    EXPECTED_HASH=$(grep "$TARBALL_NAME" "${TMP_DIR}/checksums.txt" | awk '{print $1}')
    if [ -n "$EXPECTED_HASH" ]; then
        if command -v sha256sum >/dev/null 2>&1; then
            ACTUAL_HASH=$(sha256sum "${TMP_DIR}/${TARBALL_NAME}" | awk '{print $1}')
        elif command -v shasum >/dev/null 2>&1; then
            ACTUAL_HASH=$(shasum -a 256 "${TMP_DIR}/${TARBALL_NAME}" | awk '{print $1}')
        fi

        if [ -n "$ACTUAL_HASH" ]; then
            if [ "$EXPECTED_HASH" != "$ACTUAL_HASH" ]; then
                log_error "SHA-256 verification failed!"
                log_error "Expected: $EXPECTED_HASH"
                log_error "Got:      $ACTUAL_HASH"
                exit 1
            fi
            log_info "SHA-256 checksum verified: ${ACTUAL_HASH}"
        fi
    fi
fi

# 8. Unpack and Install Binary
mkdir -p "${TMP_DIR}/extracted"
tar -xzf "${TMP_DIR}/${TARBALL_NAME}" -C "${TMP_DIR}/extracted" --strip-components=1 2>/dev/null || tar -xzf "${TMP_DIR}/${TARBALL_NAME}" -C "${TMP_DIR}/extracted"

EXTRACTED_BIN="${TMP_DIR}/extracted/tidy"
if [ ! -f "$EXTRACTED_BIN" ]; then
    EXTRACTED_BIN=$(find "${TMP_DIR}/extracted" -type f -name tidy | head -n 1)
fi

if [ -z "$EXTRACTED_BIN" ] || [ ! -f "$EXTRACTED_BIN" ]; then
    log_error "Extracted archive did not contain 'tidy' binary."
    exit 1
fi

mkdir -p "$INSTALL_DIR"
cp "$EXTRACTED_BIN" "${INSTALL_DIR}/tidy"
chmod +x "${INSTALL_DIR}/tidy"

log_success "Installed tidy to ${INSTALL_DIR}/tidy"

# 9. Verify Installation and PATH
if command -v tidy >/dev/null 2>&1; then
    VERSION_OUT=$("${INSTALL_DIR}/tidy" --version 2>/dev/null || true)
    log_success "Verified: ${VERSION_OUT}"
else
    log_warn "${INSTALL_DIR} is not in your \$PATH."
    printf "\n"
    printf "  To use 'tidy' from anywhere, add this directory to your shell configuration:\n"
    printf "    ${BOLD}export PATH=\"%s:\$PATH\"${RESET}\n\n" "$INSTALL_DIR"
    printf "  For bash: echo 'export PATH=\"%s:\$PATH\"' >> ~/.bashrc\n" "$INSTALL_DIR"
    printf "  For zsh:  echo 'export PATH=\"%s:\$PATH\"' >> ~/.zshrc\n\n" "$INSTALL_DIR"
fi

printf "\n"
log_info "Quickstart tips:"
printf "  • ${BOLD}tidy init${RESET}           : Generate a starter config.toml\n"
printf "  • ${BOLD}tidy run --dry-run${RESET}  : Preview organizing without modifying files\n"
printf "  • ${BOLD}tidy watch${RESET}          : Start real-time background watcher\n"
printf "  • ${BOLD}tidy service enable${RESET} : Install and start user daemon\n"
printf "\n"

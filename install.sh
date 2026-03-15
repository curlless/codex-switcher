#!/usr/bin/env bash
# Installer for codex-switcher CLI
# Detects OS/arch, downloads the matching CLI release asset,
# and verifies it against published checksums when available.

set -euo pipefail

VERSION="${CODEX_SWITCHER_VERSION:-${CODEX_PROFILES_VERSION:-}}"
REPO="curlless/codex-switcher"
INSTALL_DIR="${CODEX_SWITCHER_INSTALL_DIR:-${CODEX_PROFILES_INSTALL_DIR:-$HOME/.local/bin}}"

if [ -t 1 ] && [ -z "${NO_COLOR:-}" ]; then
    BOLD='\033[1m'
    GREEN='\033[0;32m'
    YELLOW='\033[0;33m'
    RED='\033[0;31m'
    RESET='\033[0m'
else
    BOLD='' GREEN='' YELLOW='' RED='' RESET=''
fi

info() {
    printf "${GREEN}==>${RESET} ${BOLD}%s${RESET}\n" "$*" >&2
}

warn() {
    printf "${YELLOW}warning:${RESET} %s\n" "$*" >&2
}

error() {
    printf "${RED}error:${RESET} %s\n" "$*" >&2
    exit 1
}

need_cmd() {
    if ! command -v "$1" > /dev/null 2>&1; then
        error "need '$1' (command not found)"
    fi
}

detect_platform() {
    local os arch

    case "$(uname -s)" in
        Linux*)     os="linux" ;;
        Darwin*)    os="darwin" ;;
        MINGW*|MSYS*|CYGWIN*|Windows_NT) os="windows" ;;
        *)          error "unsupported OS: $(uname -s)" ;;
    esac

    local machine
    machine="$(uname -m)"
    case "$machine" in
        x86_64|amd64)       arch="x86_64" ;;
        aarch64|arm64)      arch="aarch64" ;;
        *)                  error "unsupported architecture: $machine" ;;
    esac

    case "$os-$arch" in
        linux-x86_64)       echo "x86_64-unknown-linux-gnu" ;;
        linux-aarch64)      echo "aarch64-unknown-linux-gnu" ;;
        darwin-x86_64)      echo "x86_64-apple-darwin" ;;
        darwin-aarch64)     echo "aarch64-apple-darwin" ;;
        windows-x86_64)     echo "x86_64-pc-windows-msvc" ;;
        *)                  error "unsupported platform: $os-$arch" ;;
    esac
}

download_file() {
    local url="$1"
    local output="$2"
    local show_progress="${3:-false}"
    
    if command -v curl > /dev/null 2>&1; then
        if [ "$show_progress" = "true" ] && [ -t 1 ]; then
            # Show progress bar if stdout is a TTY
            curl -#fL --proto '=https' --tlsv1.2 "$url" -o "$output" || return 1
        else
            curl -fsSL --proto '=https' --tlsv1.2 "$url" -o "$output" || return 1
        fi
    elif command -v wget > /dev/null 2>&1; then
        if [ "$show_progress" = "true" ] && [ -t 1 ]; then
            # Show progress bar if stdout is a TTY
            wget --https-only --secure-protocol=TLSv1_2 --show-progress "$url" -O "$output" || return 1
        else
            wget -q --https-only --secure-protocol=TLSv1_2 "$url" -O "$output" || return 1
        fi
    else
        error "need 'curl' or 'wget' to download"
    fi
}

verify_checksum() {
    local file="$1"
    local checksum_file="$2"
    local allow_missing="${3:-false}"

    local basename
    basename="$(basename "$file")"
    local expected actual

    expected="$(
        awk -v file="$basename" '
            $2 == ("release/" file) { print $1; found=1; exit }
            $2 == file { print $1; found=1; exit }
            index($2, "/" file) && !found { fallback=$1 }
            END {
                if (!found && fallback != "") {
                    print fallback
                }
            }
        ' "$checksum_file"
    )"
    if [ -z "$expected" ]; then
        if [ "$allow_missing" = "true" ]; then
            warn "checksum not found for $basename in this legacy checksum source"
            warn "continuing without verification because the selected tag predates the canonical checksum contract"
            return 0
        fi
        error "checksum not found for $basename in checksum file"
    fi

    if command -v sha256sum > /dev/null 2>&1; then
        actual="$(sha256sum "$file" | awk '{print $1}')"
    elif command -v shasum > /dev/null 2>&1; then
        actual="$(shasum -a 256 "$file" | awk '{print $1}')"
    else
        warn "sha256sum/shasum not found, skipping checksum verification"
        return 0
    fi

    if [ "$expected" != "$actual" ]; then
        error "checksum mismatch!\n  expected: $expected\n  actual:   $actual"
    fi

    info "Checksum verified"
}

cleanup() {
    if [ -n "${TMPDIR_INSTALL:-}" ] && [ -d "$TMPDIR_INSTALL" ]; then
        rm -rf "$TMPDIR_INSTALL"
    fi
}

resolve_latest_release_asset() {
    local asset_name="$1"
    local latest_url="https://github.com/$REPO/releases/latest/download/$asset_name"

    if command -v curl > /dev/null 2>&1; then
        curl -fsSIL -o /dev/null -w '%{url_effective}' "$latest_url"
        return
    fi

    if command -v wget > /dev/null 2>&1; then
        wget -qSO- --max-redirect=20 "$latest_url" 2>&1 \
            | awk '/^  Location: / { print $2 }' \
            | tail -1 \
            | tr -d '\r'
        return
    fi

    error "need 'curl' or 'wget' to resolve the latest GitHub release"
}

main() {
    need_cmd uname
    need_cmd mkdir
    need_cmd chmod
    
    info "Installing codex-switcher CLI"
    
    local target
    target="$(detect_platform)"
    info "Detected platform: $target"
    
    local archive_name="codex-switcher-${target}.tar.gz"
    if [[ "$target" == *"windows"* ]]; then
        archive_name="codex-switcher-${target}.exe.zip"
    fi
    local archive_url release_checksum_url repo_checksum_url effective_version
    if [ -n "$VERSION" ]; then
        effective_version="${VERSION#v}"
        local base_url="https://github.com/$REPO/releases/download/v$effective_version"
        archive_url="$base_url/$archive_name"
        release_checksum_url="$base_url/SHA256SUMS"
        repo_checksum_url="https://raw.githubusercontent.com/$REPO/main/checksums/v${effective_version}.txt"
        info "Requested release: v$effective_version"
    else
        archive_url="$(resolve_latest_release_asset "$archive_name")"
        if [[ -z "$archive_url" || "$archive_url" != *"/releases/download/"* ]]; then
            error "latest release does not publish the CLI asset '$archive_name' yet"
        fi
        effective_version="$(printf '%s' "$archive_url" | sed -E 's#^.*/download/v([^/]+)/.*#\1#')"
        release_checksum_url="https://github.com/$REPO/releases/download/v${effective_version}/SHA256SUMS"
        repo_checksum_url=""
        info "Resolved latest release tag for CLI install: v$effective_version"
    fi
    
    TMPDIR_INSTALL="$(mktemp -d)"
    trap cleanup EXIT
    local tmpdir="$TMPDIR_INSTALL"
    
    local archive_path="$tmpdir/$archive_name"
    local checksum_path="$tmpdir/checksums.txt"
    
    info "Downloading binary..."
    if ! download_file "$archive_url" "$archive_path" "true"; then
        error "failed to download CLI release asset from $archive_url. This release may not publish the CLI surface for $target yet."
    fi
    
    info "Downloading checksums..."
    if download_file "$release_checksum_url" "$checksum_path" "false"; then
        info "Using release SHA256SUMS"
        verify_checksum "$archive_path" "$checksum_path"
    elif [ -n "$repo_checksum_url" ] && download_file "$repo_checksum_url" "$checksum_path" "false"; then
        warn "Release SHA256SUMS not available, falling back to repository checksums"
        verify_checksum "$archive_path" "$checksum_path" "true"
    else
        warn "Could not download checksum file from release or repo"
        warn "Proceeding without verification (not recommended)"
    fi
    
    info "Extracting..."
    if [[ "$target" == *"windows"* ]]; then
        if command -v unzip > /dev/null 2>&1; then
            unzip -q "$archive_path" -d "$tmpdir" || error "zip extraction failed"
        elif command -v python > /dev/null 2>&1; then
            python - "$archive_path" "$tmpdir" <<'PY' || error "zip extraction failed"
import sys
import zipfile

archive, out_dir = sys.argv[1], sys.argv[2]
with zipfile.ZipFile(archive) as zf:
    zf.extractall(out_dir)
PY
        elif command -v powershell.exe > /dev/null 2>&1; then
            powershell.exe -NoProfile -Command "Expand-Archive -Path '$archive_path' -DestinationPath '$tmpdir' -Force" \
                || error "zip extraction failed"
        else
            error "need 'unzip', 'python', or 'powershell.exe' to extract Windows zip releases"
        fi
    else
        need_cmd tar
        tar -xzf "$archive_path" -C "$tmpdir" || error "tar extraction failed"
    fi
    
    # Determine binary name based on OS
    local binary_name="codex-switcher"
    if [[ "$target" == *"windows"* ]]; then
        binary_name="codex-switcher.exe"
    fi
    
    local binary_path
    if [ -f "$tmpdir/$binary_name" ]; then
        binary_path="$tmpdir/$binary_name"
    elif [ -f "$tmpdir/codex-switcher/$binary_name" ]; then
        binary_path="$tmpdir/codex-switcher/$binary_name"
    else
        error "binary not found in archive (looking for $binary_name)"
    fi
    
    info "Installing to $INSTALL_DIR/$binary_name"
    mkdir -p "$INSTALL_DIR"
    
    if [ -f "$INSTALL_DIR/$binary_name" ]; then
        local backup
        backup="$INSTALL_DIR/$binary_name.backup.$(date +%s)"
        mv "$INSTALL_DIR/$binary_name" "$backup"
        info "Backed up existing binary to $backup"
    fi
    
    cp "$binary_path" "$INSTALL_DIR/$binary_name"
    
    # Make executable on Unix-like systems (not needed on Windows)
    if [[ "$target" != *"windows"* ]]; then
        chmod +x "$INSTALL_DIR/$binary_name"
    fi
    
    if [ -f "$INSTALL_DIR/$binary_name" ]; then
        local installed_version
        installed_version="$("$INSTALL_DIR/$binary_name" --version 2>&1 || echo "unknown")"
        installed_version="$(echo "$installed_version" | head -1)"
        info "Successfully installed: $installed_version"
    else
        error "installation failed: binary is not executable"
    fi
    
    if ! echo "$PATH" | grep -q "$INSTALL_DIR"; then
        warn "$INSTALL_DIR is not in your PATH"
        if [[ "$target" == *"windows"* ]]; then
            warn "Add this directory to your PATH environment variable"
            warn "Or run: setx PATH \"%PATH%;$INSTALL_DIR\""
        else
            warn "Add this to your shell profile (~/.bashrc, ~/.zshrc, etc.):"
            echo "  export PATH=\"\$PATH:$INSTALL_DIR\""
        fi
    else
        info "Installation complete! Run: $binary_name --help"
    fi
}

usage() {
    cat <<EOF
Usage: $0 [OPTIONS]

Install the codex-switcher CLI by downloading the matching GitHub Release asset for your platform.

Options:
  -v, --version VERSION    Install a specific version tag
  -d, --dir DIR            Install to directory (default: $INSTALL_DIR)
  -h, --help               Show this help message

Environment variables:
  CODEX_SWITCHER_VERSION          Override default version
  CODEX_SWITCHER_INSTALL_DIR      Override default install directory
  CODEX_PROFILES_VERSION          Legacy alias for CODEX_SWITCHER_VERSION
  CODEX_PROFILES_INSTALL_DIR      Legacy alias for CODEX_SWITCHER_INSTALL_DIR
  NO_COLOR                        Disable colored output

Notes:
  - Without --version, the installer targets the latest GitHub Release tag and
    expects that tag to publish the matching CLI asset for your platform.
  - Windows desktop GUI installers (.exe/.msi) are a separate installation
    surface and are not installed by this script.
  - Checksums are fetched from the GitHub release when available, with a
    repository fallback for older tags.

Examples:
  $0                              # Install the latest release that has your CLI asset
  $0 --version 0.2.0             # Install specific version
  $0 --dir /usr/local/bin        # Install to custom directory

EOF
}

while [[ $# -gt 0 ]]; do
    case "$1" in
        -v|--version)
            VERSION="${2#v}"
            shift 2
            ;;
        -d|--dir)
            INSTALL_DIR="$2"
            shift 2
            ;;
        -h|--help)
            usage
            exit 0
            ;;
        *)
            error "unknown option: $1\nRun '$0 --help' for usage."
            ;;
    esac
done

main

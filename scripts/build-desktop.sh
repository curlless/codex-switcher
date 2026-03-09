#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat <<'EOF'
Usage: scripts/build-desktop.sh [target] [release|debug]

Build the Tauri desktop bundle from apps/desktop and print the expected bundle output path.

Examples:
  scripts/build-desktop.sh
  scripts/build-desktop.sh x86_64-pc-windows-msvc
  scripts/build-desktop.sh x86_64-pc-windows-msvc debug
EOF
}

echo "=== Codex Switcher Desktop Build ==="
echo ""

normalize_command_path() {
  local path="$1"
  path="${path//$'\r'/}"

  if command -v wslpath >/dev/null 2>&1 && [[ "${path}" =~ ^[A-Za-z]:\\ ]]; then
    wslpath -u "${path}"
    return
  fi

  printf '%s\n' "${path}"
}

resolve_command() {
  local name="$1"
  local resolved=""

  if resolved=$(command -v "${name}" 2>/dev/null); then
    printf '%s\n' "${resolved}"
    return 0
  fi

  if command -v where.exe >/dev/null 2>&1; then
    resolved=$(where.exe "${name}" 2>/dev/null | tr -d '\r' | head -n 1)
    if [[ -n "${resolved}" ]]; then
      normalize_command_path "${resolved}"
      return 0
    fi
  fi

  return 1
}

check_command() {
  local name="$1"
  local install_hint="$2"
  local resolved=""

  if ! resolved=$(resolve_command "${name}"); then
    echo "ERROR: ${name} is not installed."
    echo "  ${install_hint}"
    exit 1
  fi

  printf '%s\n' "${resolved}"
}

if [[ "${1:-}" == "-h" || "${1:-}" == "--help" ]]; then
  usage
  exit 0
fi

RUSTC_BIN=$(check_command "rustc" "Install from https://rustup.rs")
CARGO_BIN=$(check_command "cargo" "Install from https://rustup.rs")
NODE_BIN=$(check_command "node" "Install Node.js 20+ from https://nodejs.org")
NPM_BIN=$(check_command "npm" "Install Node.js 20+ from https://nodejs.org")
NPX_BIN=$(check_command "npx" "Install Node.js 20+ from https://nodejs.org")

node_version=$("${NODE_BIN}" -v | sed 's/v//' | cut -d. -f1)
if [[ "${node_version}" -lt 20 ]]; then
  echo "ERROR: Node.js 20+ required (found $("${NODE_BIN}" -v))"
  exit 1
fi

echo "Rust:   $("${RUSTC_BIN}" --version)"
echo "Cargo:  $("${CARGO_BIN}" --version)"
echo "Node:   $("${NODE_BIN}" --version)"
echo "npm:    $("${NPM_BIN}" --version)"
echo ""

target="${1:-}"
profile="${2:-release}"

if [[ $# -gt 2 ]]; then
  usage
  exit 1
fi

case "${profile}" in
  release|debug) ;;
  *)
    echo "ERROR: unsupported profile '${profile}'. Use 'release' or 'debug'."
    exit 1
    ;;
esac

cd "$(dirname "$0")/../apps/desktop"

if [[ -n "${target}" && "${target}" != *windows* ]]; then
  echo "WARNING: US005 only accepts Windows packaging evidence."
  echo "         Continuing with non-Windows target '${target}', but it will not satisfy the story smoke boundary."
  echo ""
fi

echo "Packaging scope:"
echo "  - bundle targets from src-tauri/tauri.conf.json: nsis, msi"
echo "  - updater artifacts: disabled"
echo "  - code signing: not configured in this story"
echo ""

echo "Installing frontend dependencies..."
"${NPM_BIN}" ci

if ! "${NPX_BIN}" tauri --version >/dev/null 2>&1; then
  echo "ERROR: @tauri-apps/cli not found in devDependencies."
  echo "  Run: npm install --save-dev @tauri-apps/cli"
  exit 1
fi

echo "Tauri:  $("${NPX_BIN}" tauri --version)"
echo ""

build_args=()
if [[ -n "${target}" ]]; then
  build_args+=(--target "${target}")
fi
if [[ "${profile}" == "debug" ]]; then
  build_args+=(--debug)
fi

echo "Working directory: $(pwd)"
if [[ ${#build_args[@]} -gt 0 ]]; then
  echo "Build command: npm run tauri:build -- ${build_args[*]}"
else
  echo "Build command: npm run tauri:build"
fi
echo ""

if [[ ${#build_args[@]} -gt 0 ]]; then
  "${NPM_BIN}" run tauri:build -- "${build_args[@]}"
else
  "${NPM_BIN}" run tauri:build
fi

echo ""
echo "=== Build complete ==="
echo ""
echo "Artifacts:"

bundle_dir="src-tauri/target"
if [[ -n "${target}" ]]; then
  bundle_dir="${bundle_dir}/${target}"
fi
bundle_dir="${bundle_dir}/${profile}/bundle"

if [[ -d "${bundle_dir}" ]]; then
  echo "  Bundle directory: ${bundle_dir}"
  find "${bundle_dir}" -type f \( -name "*.exe" -o -name "*.msi" -o -name "*.dmg" -o -name "*.deb" -o -name "*.AppImage" \) | sort
else
  echo "  Bundle directory not found at ${bundle_dir}"
  echo "  Check src-tauri/target/ for build output."
fi

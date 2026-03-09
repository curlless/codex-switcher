#!/usr/bin/env bash
set -euo pipefail

echo "=== Codex Switcher Desktop Build ==="
echo ""

check_command() {
  if ! command -v "$1" >/dev/null 2>&1; then
    echo "ERROR: $1 is not installed."
    echo "  $2"
    exit 1
  fi
}

check_command "rustc" "Install from https://rustup.rs"
check_command "cargo" "Install from https://rustup.rs"
check_command "node"  "Install Node.js 22+ from https://nodejs.org"
check_command "npm"   "Install Node.js 22+ from https://nodejs.org"

node_version=$(node -v | sed 's/v//' | cut -d. -f1)
if [[ "${node_version}" -lt 20 ]]; then
  echo "ERROR: Node.js 20+ required (found $(node -v))"
  exit 1
fi

echo "Rust:   $(rustc --version)"
echo "Cargo:  $(cargo --version)"
echo "Node:   $(node --version)"
echo "npm:    $(npm --version)"
echo ""

target="${1:-}"
profile="${2:-release}"

cd "$(dirname "$0")/../apps/desktop"

echo "Installing frontend dependencies..."
npm ci

if ! npx tauri --version >/dev/null 2>&1; then
  echo "ERROR: @tauri-apps/cli not found in devDependencies."
  echo "  Run: npm install --save-dev @tauri-apps/cli"
  exit 1
fi

echo "Tauri:  $(npx tauri --version)"
echo ""

if [[ -n "${target}" ]]; then
  echo "Building desktop app for target: ${target} (${profile})..."
  npx tauri build --target "${target}"
else
  echo "Building desktop app for host platform (${profile})..."
  npx tauri build
fi

echo ""
echo "=== Build complete ==="
echo ""
echo "Artifacts:"

bundle_dir="src-tauri/target"
if [[ -n "${target}" ]]; then
  bundle_dir="${bundle_dir}/${target}"
fi
bundle_dir="${bundle_dir}/release/bundle"

if [[ -d "${bundle_dir}" ]]; then
  find "${bundle_dir}" -type f \( -name "*.exe" -o -name "*.msi" -o -name "*.dmg" -o -name "*.deb" -o -name "*.AppImage" \) | sort
else
  echo "  Bundle directory not found at ${bundle_dir}"
  echo "  Check src-tauri/target/ for build output."
fi

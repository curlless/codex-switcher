#!/usr/bin/env bash
set -euo pipefail

version="${1:-}"
out_dir="${2:-dist}"

if [[ -z "${version}" ]]; then
  version=$(python3 - <<'PY'
import json
with open("package.json", "r", encoding="utf-8") as fh:
    print(json.load(fh)["version"])
PY
)
fi

release_dir="${out_dir}/release"
npm_packages_dir="${out_dir}/npm-packages"
homebrew_dir="${out_dir}/homebrew"
cargo_dir="${out_dir}/cargo"
checksums_file="${out_dir}/checksums/SHA256SUMS"

expected_npm_package_for_target() {
  case "$1" in
    x86_64-unknown-linux-gnu) echo "1voin1-codex-switcher-linux-x64" ;;
    aarch64-unknown-linux-gnu) echo "1voin1-codex-switcher-linux-arm64" ;;
    x86_64-apple-darwin) echo "1voin1-codex-switcher-darwin-x64" ;;
    aarch64-apple-darwin) echo "1voin1-codex-switcher-darwin-arm64" ;;
    x86_64-pc-windows-msvc) echo "1voin1-codex-switcher-win32-x64" ;;
    *) return 1 ;;
  esac
}

if [[ ! -d "${release_dir}" ]]; then
  echo "Missing release dir: ${release_dir}" >&2
  exit 1
fi

if [[ ! -d "${npm_packages_dir}" ]]; then
  echo "Missing npm packages dir: ${npm_packages_dir}" >&2
  exit 1
fi

if [[ ! -d "${cargo_dir}" ]]; then
  echo "Missing cargo dir: ${cargo_dir}" >&2
  exit 1
fi

if [[ ! -f "${checksums_file}" ]]; then
  echo "Missing checksums file: ${checksums_file}" >&2
  exit 1
fi

has_release_assets=0
has_platform_npm_packages=0
shopt -s nullglob
for artifact_dir in "${out_dir}/artifacts"/codex-switcher-*; do
  target="${artifact_dir##*/codex-switcher-}"
  if [[ "${target}" == *windows* ]]; then
    expected="${release_dir}/codex-switcher-${target}.exe.zip"
  else
    expected="${release_dir}/codex-switcher-${target}.tar.gz"
  fi
  if [[ ! -f "${expected}" ]]; then
    echo "Missing release asset: ${expected}" >&2
    exit 1
  fi

  if pkg_name="$(expected_npm_package_for_target "${target}")"; then
    pkg_tgz="${npm_packages_dir}/${pkg_name}-${version}.tgz"
    if [[ ! -f "${pkg_tgz}" ]]; then
      echo "Missing platform npm package: ${pkg_tgz}" >&2
      exit 1
    fi
    has_platform_npm_packages=1
  fi

  has_release_assets=1
done
shopt -u nullglob

if [[ "${has_release_assets}" -eq 0 ]]; then
  echo "No build artifacts found under ${out_dir}/artifacts" >&2
  exit 1
fi

if [[ "${has_platform_npm_packages}" -eq 0 ]]; then
  echo "No platform npm packages found under ${npm_packages_dir}" >&2
  exit 1
fi

main_pkg="${npm_packages_dir}/1voin1-codex-switcher-${version}.tgz"
if [[ ! -f "${main_pkg}" ]]; then
  echo "Missing npm main package: ${main_pkg}" >&2
  exit 1
fi

crate="${cargo_dir}/codex-switcher-${version}.crate"
if [[ ! -f "${crate}" ]]; then
  echo "Missing cargo crate: ${crate}" >&2
  exit 1
fi

if [[ -f "${release_dir}/codex-switcher-aarch64-apple-darwin.tar.gz" || \
      -f "${release_dir}/codex-switcher-x86_64-apple-darwin.tar.gz" ]]; then
  if [[ ! -f "${homebrew_dir}/codex-switcher.rb" ]]; then
    echo "Missing Homebrew cask: ${homebrew_dir}/codex-switcher.rb" >&2
    exit 1
  fi
fi

if [[ ! -s "${checksums_file}" ]]; then
  echo "Checksums file is empty: ${checksums_file}" >&2
  exit 1
fi

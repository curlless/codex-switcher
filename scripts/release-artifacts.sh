#!/usr/bin/env bash
set -euo pipefail

version="${1:-}"
artifacts_dir="${2:-dist/artifacts}"
out_dir="${3:-dist}"

usage() {
  cat <<'EOF'
Usage: scripts/release-artifacts.sh <version> [artifacts_dir] [out_dir]

Builds release assets, npm packages, cargo crate, and Homebrew cask file
from pre-built binaries in artifacts_dir.
EOF
}

if [[ -z "${version}" ]]; then
  usage >&2
  exit 1
fi

release_dir="${out_dir}/release"
npm_dir="${out_dir}/npm"
npm_packages_dir="${out_dir}/npm-packages"
homebrew_dir="${out_dir}/homebrew"
cargo_dir="${out_dir}/cargo"
checksums_dir="${out_dir}/checksums"

rm -rf "${release_dir}" "${npm_dir}" "${npm_packages_dir}" "${homebrew_dir}" "${cargo_dir}" "${checksums_dir}"
mkdir -p "${release_dir}" "${npm_packages_dir}" "${homebrew_dir}" "${cargo_dir}" "${checksums_dir}"

# Convert to absolute paths for use in subshells
release_dir="$(cd "${release_dir}" && pwd)"
npm_packages_dir="$(cd "${npm_packages_dir}" && pwd)"
homebrew_dir="$(cd "${homebrew_dir}" && pwd)"
cargo_dir="$(cd "${cargo_dir}" && pwd)"
checksums_dir="$(cd "${checksums_dir}" && pwd)"

sha256_file() {
  if command -v sha256sum >/dev/null 2>&1; then
    sha256sum "$1" | awk '{print $1}'
  elif command -v shasum >/dev/null 2>&1; then
    shasum -a 256 "$1" | awk '{print $1}'
  else
    echo "Missing sha256sum/shasum" >&2
    exit 1
  fi
}

for artifact_dir in "${artifacts_dir}"/codex-switcher-*; do
  target="${artifact_dir##*/codex-switcher-}"
  binary="codex-switcher"
  if [[ "${target}" == *windows* ]]; then
    binary="codex-switcher.exe"
  fi

  if [[ "${target}" == *windows* ]]; then
    (cd "${artifact_dir}" && zip -j "${release_dir}/codex-switcher-${target}.exe.zip" "${binary}")
  else
    tar -C "${artifact_dir}" -czf "${release_dir}/codex-switcher-${target}.tar.gz" "${binary}"
  fi
done

scripts/package-npm.sh "${version}" "${artifacts_dir}" "${npm_dir}"
for pkg_dir in "${npm_dir}"/*; do
  npm pack "${pkg_dir}" --pack-destination "${npm_packages_dir}"
done
npm pack --pack-destination "${npm_packages_dir}"

cargo package --locked
crate_path="target/package/codex-switcher-${version}.crate"
if [[ ! -f "${crate_path}" ]]; then
  echo "Missing crate package at ${crate_path}" >&2
  exit 1
fi
cp "${crate_path}" "${cargo_dir}/"

darwin_x64="${release_dir}/codex-switcher-x86_64-apple-darwin.tar.gz"
darwin_arm="${release_dir}/codex-switcher-aarch64-apple-darwin.tar.gz"
if [[ -f "${darwin_x64}" && -f "${darwin_arm}" ]]; then
  darwin_x64_sha="$(sha256_file "${darwin_x64}")"
  darwin_arm_sha="$(sha256_file "${darwin_arm}")"
  cat > "${homebrew_dir}/codex-switcher.rb" <<EOF
cask "codex-switcher" do
  version "${version}"

  on_arm do
    sha256 "${darwin_arm_sha}"
    url "https://github.com/1Voin1/codex-switcher/releases/download/v#{version}/codex-switcher-aarch64-apple-darwin.tar.gz"
  end

  on_intel do
    sha256 "${darwin_x64_sha}"
    url "https://github.com/1Voin1/codex-switcher/releases/download/v#{version}/codex-switcher-x86_64-apple-darwin.tar.gz"
  end

  name "codex-switcher"
  desc "Manage multiple Codex CLI auth profiles"
  homepage "https://github.com/1Voin1/codex-switcher"

  binary "codex-switcher"
end
EOF
else
  echo "Skipping Homebrew cask generation; missing darwin release assets." >&2
fi

echo "Release assets:"
ls -la "${release_dir}" || true
echo "NPM package tarballs:"
ls -la "${npm_packages_dir}" || true
echo "Cargo crate:"
ls -la "${cargo_dir}" || true
echo "Homebrew cask:"
ls -la "${homebrew_dir}" || true

checksums_file="${checksums_dir}/SHA256SUMS"
: > "${checksums_file}"
shopt -s nullglob
files=(
  "${release_dir}"/*
  "${npm_packages_dir}"/*.tgz
  "${cargo_dir}"/*.crate
  "${homebrew_dir}"/*.rb
)
for file in "${files[@]}"; do
  rel_path="${file#${out_dir}/}"
  printf "%s  %s\n" "$(sha256_file "${file}")" "${rel_path}" >> "${checksums_file}"
done
shopt -u nullglob
echo "Checksums:"
ls -la "${checksums_dir}" || true

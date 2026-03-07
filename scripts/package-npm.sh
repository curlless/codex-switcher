#!/usr/bin/env bash
set -euo pipefail

version="${1:-}"
artifacts_dir="${2:-dist/artifacts}"
out_dir="${3:-dist/npm}"

if [[ -z "${version}" ]]; then
  echo "Usage: $0 <version> [artifacts_dir] [out_dir]" >&2
  exit 1
fi

rm -rf "${out_dir}"
mkdir -p "${out_dir}"

for artifact_dir in "${artifacts_dir}"/codex-switcher-*; do
  target="${artifact_dir##*/codex-switcher-}"
  pkg=""
  os=""
  cpu=""
  bin_name="codex-switcher"

  case "${target}" in
    x86_64-unknown-linux-gnu)
      pkg="@1voin1/codex-switcher-linux-x64"
      os="linux"
      cpu="x64"
      ;;
    aarch64-unknown-linux-gnu)
      pkg="@1voin1/codex-switcher-linux-arm64"
      os="linux"
      cpu="arm64"
      ;;
    x86_64-apple-darwin)
      pkg="@1voin1/codex-switcher-darwin-x64"
      os="darwin"
      cpu="x64"
      ;;
    aarch64-apple-darwin)
      pkg="@1voin1/codex-switcher-darwin-arm64"
      os="darwin"
      cpu="arm64"
      ;;
    x86_64-pc-windows-msvc)
      pkg="@1voin1/codex-switcher-win32-x64"
      os="win32"
      cpu="x64"
      bin_name="codex-switcher.exe"
      ;;
    *)
      echo "Skipping unsupported target ${target}" >&2
      continue
      ;;
  esac

  pkg_dir="${out_dir}/${pkg}"
  mkdir -p "${pkg_dir}/bin"
  cp "${artifact_dir}/${bin_name}" "${pkg_dir}/bin/${bin_name}"
  if [[ "${bin_name}" != *".exe" ]]; then
    chmod +x "${pkg_dir}/bin/${bin_name}"
  fi

  cat > "${pkg_dir}/package.json" <<JSON
{
  "name": "${pkg}",
  "version": "${version}",
  "license": "MIT",
  "os": ["${os}"],
  "cpu": ["${cpu}"],
  "files": ["bin"],
  "description": "Platform binary for codex-switcher"
}
JSON
done

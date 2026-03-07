#!/usr/bin/env node

import { spawnSync } from "node:child_process";
import { readFileSync } from "node:fs";
import https from "node:https";

const args = process.argv.slice(2);
let requestedVersion = null;
let repo = "1Voin1/codex-switcher";
let requireRegistries = false;

for (let i = 0; i < args.length; i += 1) {
  const arg = args[i];
  if (arg === "--repo") {
    repo = args[i + 1];
    i += 1;
    continue;
  }
  if (arg === "--require-registries") {
    requireRegistries = true;
    continue;
  }
  if (!requestedVersion) {
    requestedVersion = arg;
    continue;
  }
  throw new Error(`unexpected argument: ${arg}`);
}

const pkg = JSON.parse(readFileSync("package.json", "utf8"));
const cargoToml = readFileSync("Cargo.toml", "utf8");
const crateNameMatch = cargoToml.match(/^name\s*=\s*"([^"]+)"/m);

if (!crateNameMatch) {
  throw new Error("failed to read crate name from Cargo.toml");
}

const crateName = crateNameMatch[1];
const version = requestedVersion ?? pkg.version;
const tag = version.startsWith("v") ? version : `v${version}`;
const normalizedVersion = tag.slice(1);

function usage() {
  process.stderr.write(
    "Usage: node scripts/verify-release-publication.mjs [version|vX.Y.Z] [--repo owner/name] [--require-registries]\n"
  );
}

function runGh(argv) {
  const result = spawnSync("gh", argv, {
    encoding: "utf8",
    stdio: ["ignore", "pipe", "pipe"],
  });

  if (result.error) {
    throw new Error(`failed to execute gh ${argv.join(" ")}: ${result.error.message}`);
  }

  if (result.status !== 0) {
    const stderr = (result.stderr || result.stdout || "").trim();
    throw new Error(`gh ${argv.join(" ")} failed: ${stderr}`);
  }

  return result.stdout;
}

function getJson(url) {
  return new Promise((resolve, reject) => {
    https
      .get(
        url,
        {
          headers: {
            "User-Agent": "codex-switcher-release-verifier",
            Accept: "application/json",
          },
        },
        (response) => {
          const chunks = [];
          response.on("data", (chunk) => chunks.push(chunk));
          response.on("end", () => {
            const body = Buffer.concat(chunks).toString("utf8");
            if ((response.statusCode ?? 500) >= 400) {
              reject(
                new Error(`${url} returned ${response.statusCode}: ${body.slice(0, 200)}`)
              );
              return;
            }

            try {
              resolve(JSON.parse(body));
            } catch (error) {
              reject(new Error(`invalid JSON from ${url}: ${error}`));
            }
          });
        }
      )
      .on("error", reject);
  });
}

function packFileName(packageName, packageVersion) {
  return `${packageName.replace(/^@/, "").replace(/\//g, "-")}-${packageVersion}.tgz`;
}

function baseReleaseAssets(packageVersion) {
  return [
    `${crateName}-${packageVersion}.crate`,
    "SHA256SUMS",
    "codex-switcher.rb",
    "codex-switcher-aarch64-apple-darwin.tar.gz",
    "codex-switcher-aarch64-unknown-linux-gnu.tar.gz",
    "codex-switcher-x86_64-apple-darwin.tar.gz",
    "codex-switcher-x86_64-pc-windows-msvc.exe.zip",
    "codex-switcher-x86_64-unknown-linux-gnu.tar.gz",
  ].sort();
}

function expectedReleaseAssets(packageVersion) {
  const scopedNpmAssets = [
    packFileName(pkg.name, packageVersion),
    ...Object.keys(pkg.optionalDependencies ?? {}).map((name) => packFileName(name, packageVersion)),
  ].sort();

  const legacyNpmAssets = [
    `codex-switcher-${packageVersion}.tgz`,
    `codex-switcher-darwin-arm64-${packageVersion}.tgz`,
    `codex-switcher-darwin-x64-${packageVersion}.tgz`,
    `codex-switcher-linux-arm64-${packageVersion}.tgz`,
    `codex-switcher-linux-x64-${packageVersion}.tgz`,
    `codex-switcher-win32-x64-${packageVersion}.tgz`,
  ].sort();

  return {
    scoped: [...baseReleaseAssets(packageVersion), ...scopedNpmAssets].sort(),
    legacy: [...baseReleaseAssets(packageVersion), ...legacyNpmAssets].sort(),
  };
}

function printLine(status, label, detail) {
  process.stdout.write(`${status.padEnd(7)} ${label}: ${detail}\n`);
}

function registryVersionStatus(actualVersion, expectedVersion) {
  if (actualVersion === expectedVersion) {
    return { ok: true, detail: actualVersion };
  }

  if (!actualVersion) {
    return { ok: false, detail: "not published" };
  }

  return { ok: false, detail: `found ${actualVersion}, expected ${expectedVersion}` };
}

async function main() {
  if (args.includes("--help") || args.includes("-h")) {
    usage();
    process.exit(0);
  }

  const release = JSON.parse(
    runGh(["release", "view", tag, "-R", repo, "--json", "tagName,isDraft,isPrerelease,assets,url"])
  );
  const releaseAssets = new Set(release.assets.map((asset) => asset.name));
  const expectedAssets = expectedReleaseAssets(normalizedVersion);
  const missingScopedAssets = expectedAssets.scoped.filter((name) => !releaseAssets.has(name));
  const missingLegacyAssets = expectedAssets.legacy.filter((name) => !releaseAssets.has(name));
  const assetMode =
    missingScopedAssets.length === 0
      ? "scoped"
      : missingLegacyAssets.length === 0
        ? "legacy-unscoped"
        : null;

  printLine("PASS", "GitHub release", release.url);
  printLine("PASS", "Tag", release.tagName);
  printLine("PASS", "Draft", String(release.isDraft));
  printLine("PASS", "Prerelease", String(release.isPrerelease));

  if (!assetMode) {
    printLine(
      "FAIL",
      "Release assets",
      `missing scoped assets ${missingScopedAssets.join(", ")}`
    );
  } else {
    const expectedCount = assetMode === "scoped" ? expectedAssets.scoped.length : expectedAssets.legacy.length;
    const detail =
      assetMode === "scoped"
        ? `${expectedCount} expected assets present`
        : `${expectedCount} expected assets present (legacy unscoped npm tarballs)`;
    printLine("PASS", "Release assets", detail);
  }

  runGh(["api", `repos/${repo}/contents/checksums/v${normalizedVersion}.txt?ref=develop`]);
  printLine("PASS", "Checksum commit", `checksums/v${normalizedVersion}.txt exists on develop`);

  const npmPackages = [pkg.name, ...Object.keys(pkg.optionalDependencies ?? {})];
  let registryFailures = 0;

  for (const packageName of npmPackages) {
    let status;
    try {
      const npmMeta = await getJson(
        `https://registry.npmjs.org/${encodeURIComponent(packageName).replace(/%40/g, "@")}`
      );
      status = registryVersionStatus(npmMeta["dist-tags"]?.latest ?? null, normalizedVersion);
    } catch {
      status = { ok: false, detail: "not published" };
    }

    if (!status.ok) {
      registryFailures += 1;
      printLine(requireRegistries ? "FAIL" : "WARN", `npm ${packageName}`, status.detail);
    } else {
      printLine("PASS", `npm ${packageName}`, status.detail);
    }
  }

  let crateStatus;
  try {
    const cratesMeta = await getJson(`https://crates.io/api/v1/crates/${crateName}`);
    crateStatus = registryVersionStatus(cratesMeta.crate?.newest_version ?? null, normalizedVersion);
  } catch {
    crateStatus = { ok: false, detail: "not published" };
  }

  if (!crateStatus.ok) {
    registryFailures += 1;
    printLine(requireRegistries ? "FAIL" : "WARN", `crates.io ${crateName}`, crateStatus.detail);
  } else {
    printLine("PASS", `crates.io ${crateName}`, crateStatus.detail);
  }

  if (!assetMode) {
    process.exit(1);
  }

  if (requireRegistries && registryFailures > 0) {
    process.exit(1);
  }
}

main().catch((error) => {
  process.stderr.write(`${error.message}\n`);
  process.exit(1);
});

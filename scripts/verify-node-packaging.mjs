#!/usr/bin/env node

import { readFileSync, existsSync } from "node:fs";
import { spawnSync } from "node:child_process";

const pkg = JSON.parse(readFileSync("package.json", "utf8"));
const npmrc = readFileSync(".npmrc", "utf8");

if (pkg.name !== "@1voin1/codex-switcher") {
  throw new Error("package name must be @1voin1/codex-switcher");
}

if (pkg?.bin?.["codex-switcher"] !== "bin/codex-switcher.js") {
  throw new Error("bin entry must point to bin/codex-switcher.js");
}

if (!Array.isArray(pkg.files) || !pkg.files.includes("bin")) {
  throw new Error("package files must include bin/");
}

if (!existsSync("bin/codex-switcher.js")) {
  throw new Error("bin/codex-switcher.js is missing");
}

if (Object.keys(pkg.dependencies ?? {}).length !== 0) {
  throw new Error("package dependencies must stay empty for the thin wrapper package");
}

if (Object.keys(pkg.devDependencies ?? {}).length !== 0) {
  throw new Error("package devDependencies must stay empty for the thin wrapper package");
}

if (!npmrc.split(/\r?\n/).some((line) => line.trim() === "package-lock=false")) {
  throw new Error(".npmrc must disable package-lock generation for the thin wrapper package");
}

const expectedOptionalPackages = [
  "@1voin1/codex-switcher-darwin-arm64",
  "@1voin1/codex-switcher-darwin-x64",
  "@1voin1/codex-switcher-linux-arm64",
  "@1voin1/codex-switcher-linux-x64",
  "@1voin1/codex-switcher-win32-x64",
];
const optionalDependencies = pkg.optionalDependencies ?? {};
const actualOptionalPackages = Object.keys(optionalDependencies).sort();

for (const name of expectedOptionalPackages) {
  if (!actualOptionalPackages.includes(name)) {
    throw new Error(`missing optional dependency ${name}`);
  }
}

for (const name of actualOptionalPackages) {
  if (!expectedOptionalPackages.includes(name)) {
    throw new Error(`unexpected optional dependency ${name}`);
  }
  if (optionalDependencies[name] !== pkg.version) {
    throw new Error(
      `${name} version ${optionalDependencies[name]} does not match root version ${pkg.version}`
    );
  }
}

const npmPack =
  process.platform === "win32"
    ? spawnSync(
        process.env.ComSpec ?? "cmd.exe",
        ["/d", "/s", "/c", "npm pack --dry-run --json"],
        {
          encoding: "utf8",
          stdio: ["ignore", "pipe", "pipe"],
        }
      )
    : spawnSync("npm", ["pack", "--dry-run", "--json"], {
        encoding: "utf8",
        stdio: ["ignore", "pipe", "pipe"],
      });

if (npmPack.error) {
  throw new Error(`failed to execute npm pack --dry-run --json: ${npmPack.error.message}`);
}

if (npmPack.status !== 0) {
  const output = npmPack.stderr ?? npmPack.stdout ?? "";
  process.stderr.write(String(output));
  process.exit(npmPack.status ?? 1);
}

try {
  JSON.parse(npmPack.stdout);
} catch (error) {
  throw new Error(`npm pack --dry-run did not return valid JSON: ${error}`);
}

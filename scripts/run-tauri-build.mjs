import { spawnSync } from "node:child_process";
import path from "node:path";
import { fileURLToPath } from "node:url";

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const repoRoot = path.resolve(__dirname, "..");
const desktopRoot = process.cwd();

function run(command, args, options = {}) {
  const result = spawnSync(command, args, {
    cwd: options.cwd ?? desktopRoot,
    stdio: "inherit",
    shell: options.shell ?? false,
    env: process.env,
  });

  if (result.error) {
    throw result.error;
  }

  if (typeof result.status === "number" && result.status !== 0) {
    process.exit(result.status);
  }
}

if (process.platform === "win32") {
  const prepareScript = path.join(repoRoot, "scripts", "prepare-tauri-bundler-tools.ps1");
  run("powershell", ["-ExecutionPolicy", "Bypass", "-File", prepareScript]);
}

if (process.platform === "win32") {
  const tauriCommand = path.join(desktopRoot, "node_modules", ".bin", "tauri.cmd");
  const quotedArgs = ["build", ...process.argv.slice(2)].map((arg) => `"${arg.replaceAll('"', '\\"')}"`);
  run(`"${tauriCommand}" ${quotedArgs.join(" ")}`, [], { shell: true });
} else {
  const tauriCommand = path.join(desktopRoot, "node_modules", ".bin", "tauri");
  run(tauriCommand, ["build", ...process.argv.slice(2)]);
}

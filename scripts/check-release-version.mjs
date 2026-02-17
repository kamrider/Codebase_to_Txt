import { readFileSync } from "node:fs";
import { resolve } from "node:path";

function readJson(filePath) {
  return JSON.parse(readFileSync(filePath, "utf8"));
}

function readCargoVersion(filePath) {
  const content = readFileSync(filePath, "utf8");
  const lines = content.split(/\r?\n/);
  let inPackageSection = false;

  for (const line of lines) {
    const sectionMatch = line.match(/^\s*\[(.+)\]\s*$/);
    if (sectionMatch) {
      inPackageSection = sectionMatch[1].trim() === "package";
      continue;
    }
    if (!inPackageSection) {
      continue;
    }
    const versionMatch = line.match(/^\s*version\s*=\s*"([^"]+)"\s*$/);
    if (versionMatch) {
      return versionMatch[1];
    }
  }

  throw new Error("Could not find [package].version in src-tauri/Cargo.toml");
}

const root = process.cwd();
const packageJsonPath = resolve(root, "package.json");
const tauriConfPath = resolve(root, "src-tauri", "tauri.conf.json");
const cargoTomlPath = resolve(root, "src-tauri", "Cargo.toml");

const packageVersion = readJson(packageJsonPath).version;
const tauriVersion = readJson(tauriConfPath).version;
const cargoVersion = readCargoVersion(cargoTomlPath);

const versions = {
  "package.json": packageVersion,
  "src-tauri/tauri.conf.json": tauriVersion,
  "src-tauri/Cargo.toml": cargoVersion,
};

const uniqueVersions = new Set(Object.values(versions));

if (uniqueVersions.size !== 1) {
  console.error("Release version mismatch detected:");
  for (const [file, version] of Object.entries(versions)) {
    console.error(`- ${file}: ${version}`);
  }
  process.exit(1);
}

console.log(`Release version check passed: ${packageVersion}`);

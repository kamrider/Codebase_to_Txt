import { execFileSync } from "node:child_process";
import { readFileSync } from "node:fs";
import { resolve } from "node:path";

const SEMVER_TAG_PATTERN = /^v\d+\.\d+\.\d+(?:-[0-9A-Za-z.-]+)?$/;

function readJson(filePath) {
  return JSON.parse(readFileSync(filePath, "utf8"));
}

function runGit(args) {
  return execFileSync("git", args, { encoding: "utf8" }).trim();
}

const inputTag = process.argv[2];
if (!inputTag) {
  console.error("Usage: npm run release:tag -- vX.Y.Z");
  process.exit(1);
}
if (!SEMVER_TAG_PATTERN.test(inputTag)) {
  console.error(`Invalid tag format: ${inputTag}`);
  console.error("Expected format: vX.Y.Z (optionally with pre-release suffix).");
  process.exit(1);
}

const repoRoot = process.cwd();
const packageVersion = readJson(resolve(repoRoot, "package.json")).version;
const expectedTag = `v${packageVersion}`;
if (inputTag !== expectedTag) {
  console.error(
    `Tag/version mismatch: package.json is ${packageVersion}, expected tag ${expectedTag}, got ${inputTag}.`,
  );
  process.exit(1);
}

const dirty = runGit(["status", "--porcelain"]);
if (dirty.length > 0) {
  console.error("Working tree is not clean. Commit or stash changes before tagging.");
  process.exit(1);
}

const existingTags = runGit(["tag", "--list", inputTag]);
if (existingTags === inputTag) {
  console.error(`Tag already exists: ${inputTag}`);
  process.exit(1);
}

runGit(["tag", inputTag]);
console.log(`Created tag ${inputTag}`);
console.log(`Next step: git push origin ${inputTag}`);

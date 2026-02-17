# Codebase_to_Txt

Desktop tool (Tauri + React + TypeScript) for scanning a codebase, reviewing file selection in a tree, and exporting selected content to a text file.

## Features

- Lazy-loaded directory tree with checkbox selection
- Parent/child checkbox linkage with partial state
- `.gitignore` applied during scan (not only at export time)
- Manual selection override (`include` / `exclude`) from the tree
- Selection evaluation and export preview before writing output

## Selection Behavior

- If `Apply .gitignore` is enabled, ignored files/folders are marked as `gitignored` in the tree and default to unchecked.
- Non-ignored files/folders default to checked.
- You can always manually check/uncheck any visible node to override defaults.
- Manual overrides are persisted in `manualSelections` and used by evaluation/export.

## Run

### Frontend only (Vite)

```bash
npm install
npm run dev
```

On Windows PowerShell with strict execution policy, use:

```powershell
npm.cmd install
npm.cmd run dev
```

### Tauri desktop app

```bash
npm run tauri dev
```

Prerequisites:

- Rust toolchain (`cargo`, `rustc`)
- Visual Studio C++ Build Tools (Windows)

## Build

```bash
npm run build
```

## Test (Rust backend)

```bash
cd src-tauri
cargo test
```

## Release (Windows MSI + NSIS)

This repository is configured to publish installers to GitHub Releases through GitHub Actions.

- Trigger: push a tag named `v*` (example: `v0.1.1`)
- Runner: `windows-latest`
- Bundles: `msi` and `nsis`
- Signing: disabled for now (`--no-sign`)

### 1) Ensure versions are aligned

Keep these three version values identical:

- `package.json` -> `version`
- `src-tauri/Cargo.toml` -> `[package].version`
- `src-tauri/tauri.conf.json` -> `version`

Check locally:

```bash
npm run release:check
```

### 2) Create and push the release tag

```bash
npm run release:tag -- v0.1.1
git push origin v0.1.1
```

### 3) Verify Release artifacts

After GitHub Actions finishes, verify the Release contains at least:

- one `.msi` installer
- one `.exe` installer (NSIS)

### Manual CI validation (no release publish)

You can run `Release` workflow via `workflow_dispatch` to validate build and bundling without creating a tag-based release.

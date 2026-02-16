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

import { invoke } from "@tauri-apps/api/core";
import type {
  ExportConfig,
  ExportResult,
  PreviewMeta,
  SelectionSummary,
  TreeNode,
} from "../types/export";

export async function scanTree(config: ExportConfig): Promise<TreeNode> {
  return invoke<TreeNode>("scan_tree", { config });
}

export async function scanChildren(
  config: ExportConfig,
  dirPath: string,
): Promise<TreeNode[]> {
  return invoke<TreeNode[]>("scan_children", { config, dirPath });
}

export async function evaluateSelection(
  config: ExportConfig,
): Promise<SelectionSummary> {
  return invoke<SelectionSummary>("evaluate_selection", { config });
}

export async function previewExport(config: ExportConfig): Promise<PreviewMeta> {
  return invoke<PreviewMeta>("preview_export", { config });
}

export async function runExport(
  config: ExportConfig,
  outputPath: string,
): Promise<ExportResult> {
  return invoke<ExportResult>("run_export", { config, outputPath });
}

export async function pickExportPath(defaultPath?: string): Promise<string | null> {
  return invoke<string | null>("plugin:dialog|save", {
    options: {
      defaultPath: defaultPath?.trim() ? defaultPath.trim() : undefined,
      filters: [{ name: "Text", extensions: ["txt"] }],
    },
  });
}

export async function pickRootDirectory(defaultPath?: string): Promise<string | null> {
  const result = await invoke<string | string[] | null>("plugin:dialog|open", {
    options: {
      defaultPath: defaultPath?.trim() ? defaultPath.trim() : undefined,
      directory: true,
      multiple: false,
    },
  });

  if (Array.isArray(result)) {
    return result[0] ?? null;
  }
  return result;
}

import { invoke } from "@tauri-apps/api/core";
import type {
  ExportConfig,
  ExportResult,
  PreviewMeta,
  SelectionSummary,
  TreeNode,
} from "../types/export";

export async function scanTree(rootPath: string): Promise<TreeNode> {
  return invoke<TreeNode>("scan_tree", { rootPath });
}

export async function scanChildren(
  rootPath: string,
  dirPath: string,
): Promise<TreeNode[]> {
  return invoke<TreeNode[]>("scan_children", { rootPath, dirPath });
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

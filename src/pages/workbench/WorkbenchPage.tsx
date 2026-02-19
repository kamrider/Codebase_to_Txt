import { useMemo, useState } from "react";
import { DirectoryPanel } from "../../features/explorer/components/DirectoryPanel";
import { ExportPanel } from "../../features/export/components/ExportPanel";
import { RulesPanel } from "../../features/rules/components/RulesPanel";
import {
  evaluateSelection,
  pickRootDirectory,
  pickExportPath,
  previewExport,
  runExport,
  scanChildren,
  scanTree,
} from "../../shared/api/tauriClient";
import type { ExportConfig, ExportResult, PreviewMeta, SelectionSummary, TreeNode } from "../../shared/types/export";
import { defaultExportConfig } from "../../shared/types/export";

export function WorkbenchPage() {
  const [config, setConfig] = useState<ExportConfig>(() => {
    const persistedRootPath = loadPersistedRootPath();
    if (!persistedRootPath) {
      return defaultExportConfig;
    }
    return {
      ...defaultExportConfig,
      rootPath: persistedRootPath,
    };
  });
  const [tree, setTree] = useState<TreeNode | null>(null);
  const [selectionSummary, setSelectionSummary] = useState<SelectionSummary | null>(null);
  const [preview, setPreview] = useState<PreviewMeta | null>(null);
  const [exportResult, setExportResult] = useState<ExportResult | null>(null);
  const [outputPath, setOutputPath] = useState(() => buildDefaultOutputPath(config.rootPath));
  const [pendingAction, setPendingAction] = useState<string | null>(null);
  const [errorMessage, setErrorMessage] = useState<string | null>(null);
  const [expandedPaths, setExpandedPaths] = useState<Set<string>>(new Set());
  const [loadingPaths, setLoadingPaths] = useState<Set<string>>(new Set());

  const busy = useMemo(() => pendingAction !== null, [pendingAction]);

  const runAction = async <T,>(label: string, action: () => Promise<T>): Promise<T | null> => {
    setPendingAction(label);
    setErrorMessage(null);
    try {
      return await action();
    } catch (error) {
      const text = error instanceof Error ? error.message : String(error);
      setErrorMessage(formatBackendError(text));
      return null;
    } finally {
      setPendingAction(null);
    }
  };

  const updateConfig = (patch: Partial<ExportConfig>) => {
    setConfig((previous) => ({ ...previous, ...patch }));
  };

  const handleRootPathChange = (nextPath: string) => {
    persistRootPath(nextPath);
    setConfig((previous) => ({
      ...previous,
      rootPath: nextPath,
      manualSelections: {},
    }));
    setTree(null);
    setSelectionSummary(null);
    setPreview(null);
    setExportResult(null);
    setExpandedPaths(new Set());
    setLoadingPaths(new Set());
    setErrorMessage(null);
    setOutputPath(buildDefaultOutputPath(nextPath));
  };

  const handlePickRootPath = async () => {
    const pickedPath = await runAction("pick-root-path", async () => pickRootDirectory(config.rootPath));
    if (pickedPath) {
      const normalizedPath = pickedPath.replace(/\\/g, "/");
      if (normalizedPath.trim()) {
        handleRootPathChange(normalizedPath);
      }
    }
  };

  const handleScan = async () => {
    const result = await runAction("scan", async () => scanTree(config.rootPath, config.useGitignore));
    if (result) {
      setTree(result);
      setExpandedPaths(new Set(["."]));
      setLoadingPaths(new Set());
    }
  };

  const handleToggleNode = async (node: TreeNode) => {
    if (!node.isDir) {
      return;
    }

    if (expandedPaths.has(node.path)) {
      setExpandedPaths((previous) => {
        const next = new Set(previous);
        next.delete(node.path);
        return next;
      });
      return;
    }

    setExpandedPaths((previous) => new Set(previous).add(node.path));

    if (node.children.length > 0 || node.childrenCount === 0) {
      return;
    }

    setLoadingPaths((previous) => new Set(previous).add(node.path));
    try {
      const children = await scanChildren(config.rootPath, node.path, config.useGitignore);
      setTree((previous) => {
        if (!previous) {
          return previous;
        }
        return patchTreeByPath(previous, node.path, (targetNode) => ({
          ...targetNode,
          children,
          childrenCount: children.length,
        }));
      });
    } catch (error) {
      const text = error instanceof Error ? error.message : String(error);
      setErrorMessage(formatBackendError(text));
    } finally {
      setLoadingPaths((previous) => {
        const next = new Set(previous);
        next.delete(node.path);
        return next;
      });
    }
  };

  const handleEvaluate = async () => {
    const result = await runAction("evaluate", async () => evaluateSelection(config));
    if (result) {
      setSelectionSummary(result);
    }
  };

  const handleSyncManualSelections = (
    checkedPaths: string[],
    _changedPath: string,
    _changedChecked: boolean,
  ) => {
    setConfig((previous) => {
      const loadedPaths = tree ? collectTreePaths(tree) : new Set<string>();
      const nextSelections = { ...previous.manualSelections };
      const checkedSet = new Set(checkedPaths);

      for (const path of loadedPaths) {
        if (path === ".") {
          continue;
        }
        nextSelections[path] = checkedSet.has(path) ? "include" : "exclude";
      }

      return { ...previous, manualSelections: nextSelections };
    });
    setSelectionSummary(null);
    setPreview(null);
    setExportResult(null);
  };

  const handlePreview = async () => {
    const result = await runAction("preview", async () => previewExport(config));
    if (result) {
      setPreview(result);
    }
  };

  const handleExport = async () => {
    if (!outputPath.trim()) {
      setErrorMessage("Output path is required before export.");
      return;
    }

    const evaluated = await runAction("evaluate", async () => evaluateSelection(config));
    if (!evaluated) {
      return;
    }
    setSelectionSummary(evaluated);

    if (evaluated.includedFiles === 0) {
      setErrorMessage("No files selected for export. Run Evaluate and include at least one file.");
      setExportResult(null);
      return;
    }

    const result = await runAction("export", async () => runExport(config, outputPath.trim()));
    if (result) {
      setExportResult(result);
    }
  };

  const handlePickOutputPath = async () => {
    const pickedPath = await runAction("pick-path", async () => pickExportPath(outputPath));
    if (pickedPath) {
      setOutputPath(pickedPath.replace(/\\/g, "/"));
    }
  };

  return (
    <main className="workbench">
      <DirectoryPanel
        rootPath={config.rootPath}
        useGitignore={config.useGitignore}
        busy={busy}
        tree={tree}
        selectionSummary={selectionSummary}
        expandedPaths={expandedPaths}
        loadingPaths={loadingPaths}
        manualSelections={config.manualSelections}
        onRootPathChange={handleRootPathChange}
        onPickRootPath={handlePickRootPath}
        onScan={handleScan}
        onEvaluate={handleEvaluate}
        onToggleNode={handleToggleNode}
        onSyncManualSelections={handleSyncManualSelections}
      />
      <RulesPanel config={config} onUpdateConfig={updateConfig} />
      <ExportPanel
        busy={busy}
        outputPath={outputPath}
        preview={preview}
        exportResult={exportResult}
        selectionSummary={selectionSummary}
        errorMessage={errorMessage}
        onOutputPathChange={setOutputPath}
        onPickOutputPath={handlePickOutputPath}
        onPreview={handlePreview}
        onExport={handleExport}
      />
    </main>
  );
}

function formatBackendError(raw: string): string {
  const parsed = parseCodedError(raw);
  if (!parsed) {
    return raw;
  }
  const friendly = ERROR_CODE_MESSAGES[parsed.code] ?? parsed.message ?? "Operation failed.";
  return `${friendly} [debug: ${raw}]`;
}

function parseCodedError(raw: string): { code: string; message: string } | null {
  const match = raw.trim().match(/^\[(E_[A-Z_]+)\]\s*(.*)$/);
  if (!match) {
    return null;
  }
  return { code: match[1], message: match[2] ?? "" };
}

const ERROR_CODE_MESSAGES: Record<string, string> = {
  E_ROOT_REQUIRED: "Root path is required.",
  E_ROOT_INVALID: "Root path does not exist or cannot be resolved.",
  E_ROOT_NOT_DIR: "Root path must be a directory.",
  E_PATH_OUTSIDE_ROOT: "The requested path is outside the selected root.",
  E_DIRPATH_NOT_DIR: "The requested dirPath is not a directory.",
  E_OUTPUT_REQUIRED: "Output path is required.",
  E_OUTPUT_IS_DIR: "Output path must be a file path, not a directory.",
  E_OUTPUT_EXISTS: "Output file conflict occurred.",
  E_IO_READ: "Read failed while scanning or exporting files.",
  E_IO_WRITE: "Write failed while creating export output. Check file path and write permissions.",
  E_RULE_INVALID_GLOB: "One or more glob rules are invalid.",
};

function buildDefaultOutputPath(rootPath: string): string {
  const normalizedRoot = rootPath.trim().replace(/\\/g, "/").replace(/\/+$/, "");
  const fileName = `codebase-to-txt-${formatTimestampForFileName(new Date())}.txt`;
  if (!normalizedRoot) {
    return `exports/${fileName}`;
  }
  return `${normalizedRoot}/exports/${fileName}`;
}

function formatTimestampForFileName(date: Date): string {
  const pad = (value: number) => String(value).padStart(2, "0");
  const year = date.getFullYear();
  const month = pad(date.getMonth() + 1);
  const day = pad(date.getDate());
  const hour = pad(date.getHours());
  const minute = pad(date.getMinutes());
  const second = pad(date.getSeconds());
  return `${year}${month}${day}-${hour}${minute}${second}`;
}

function patchTreeByPath(
  node: TreeNode,
  targetPath: string,
  updater: (target: TreeNode) => TreeNode,
): TreeNode {
  if (node.path === targetPath) {
    return updater(node);
  }

  if (node.children.length === 0) {
    return node;
  }

  let changed = false;
  const nextChildren = node.children.map((childNode) => {
    const patched = patchTreeByPath(childNode, targetPath, updater);
    if (patched !== childNode) {
      changed = true;
    }
    return patched;
  });

  if (!changed) {
    return node;
  }

  return { ...node, children: nextChildren };
}

function collectTreePaths(root: TreeNode): Set<string> {
  const paths = new Set<string>();

  const walk = (node: TreeNode) => {
    paths.add(node.path);
    for (const childNode of node.children) {
      walk(childNode);
    }
  };

  walk(root);
  return paths;
}

const ROOT_PATH_STORAGE_KEY = "codebase_to_txt:last_root_path";

function loadPersistedRootPath(): string {
  try {
    const stored = window.localStorage.getItem(ROOT_PATH_STORAGE_KEY);
    if (!stored) {
      return "";
    }
    return stored.trim().replace(/\\/g, "/");
  } catch {
    return "";
  }
}

function persistRootPath(nextPath: string): void {
  try {
    const normalized = nextPath.trim().replace(/\\/g, "/");
    if (!normalized) {
      window.localStorage.removeItem(ROOT_PATH_STORAGE_KEY);
      return;
    }
    window.localStorage.setItem(ROOT_PATH_STORAGE_KEY, normalized);
  } catch {
    // localStorage may be unavailable in rare host environments.
  }
}

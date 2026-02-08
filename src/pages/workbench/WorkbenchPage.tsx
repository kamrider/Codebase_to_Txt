import { useMemo, useState } from "react";
import { DirectoryPanel } from "../../features/explorer/components/DirectoryPanel";
import { ExportPanel } from "../../features/export/components/ExportPanel";
import { RulesPanel } from "../../features/rules/components/RulesPanel";
import {
  evaluateSelection,
  previewExport,
  runExport,
  scanChildren,
  scanTree,
} from "../../shared/api/tauriClient";
import type {
  ExportConfig,
  ExportResult,
  PreviewMeta,
  SelectionSummary,
  TreeNode,
} from "../../shared/types/export";
import { defaultExportConfig } from "../../shared/types/export";

export function WorkbenchPage() {
  const [config, setConfig] = useState<ExportConfig>(defaultExportConfig);
  const [tree, setTree] = useState<TreeNode | null>(null);
  const [selectionSummary, setSelectionSummary] = useState<SelectionSummary | null>(null);
  const [preview, setPreview] = useState<PreviewMeta | null>(null);
  const [exportResult, setExportResult] = useState<ExportResult | null>(null);
  const [outputPath, setOutputPath] = useState("D:/exports/codebase-to-txt-output.txt");
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
      setErrorMessage(text);
      return null;
    } finally {
      setPendingAction(null);
    }
  };

  const updateConfig = (patch: Partial<ExportConfig>) => {
    setConfig((previous) => ({ ...previous, ...patch }));
  };

  const handleRootPathChange = (nextPath: string) => {
    updateConfig({ rootPath: nextPath });
  };

  const handleScan = async () => {
    const result = await runAction("scan", async () => scanTree(config.rootPath));
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
      const children = await scanChildren(config.rootPath, node.path);
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
      setErrorMessage(text);
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

  const handlePreview = async () => {
    const result = await runAction("preview", async () => previewExport(config));
    if (result) {
      setPreview(result);
    }
  };

  const handleExport = async () => {
    const result = await runAction("export", async () => runExport(config, outputPath));
    if (result) {
      setExportResult(result);
    }
  };

  return (
    <main className="workbench">
      <DirectoryPanel
        rootPath={config.rootPath}
        busy={busy}
        tree={tree}
        selectionSummary={selectionSummary}
        expandedPaths={expandedPaths}
        loadingPaths={loadingPaths}
        onRootPathChange={handleRootPathChange}
        onScan={handleScan}
        onEvaluate={handleEvaluate}
        onToggleNode={handleToggleNode}
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
        onPreview={handlePreview}
        onExport={handleExport}
      />
    </main>
  );
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

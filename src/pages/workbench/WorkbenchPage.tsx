import { useMemo, useState } from "react";
import { DirectoryPanel } from "../../features/explorer/components/DirectoryPanel";
import { ExportPanel } from "../../features/export/components/ExportPanel";
import { RulesPanel } from "../../features/rules/components/RulesPanel";
import {
  evaluateSelection,
  previewExport,
  runExport,
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
        onRootPathChange={handleRootPathChange}
        onScan={handleScan}
        onEvaluate={handleEvaluate}
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

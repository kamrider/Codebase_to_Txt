import type {
  ExportResult,
  PreviewMeta,
  SelectionSummary,
} from "../../../shared/types/export";

type ExportPanelProps = {
  busy: boolean;
  outputPath: string;
  structureOnly: boolean;
  preview: PreviewMeta | null;
  exportResult: ExportResult | null;
  selectionSummary: SelectionSummary | null;
  errorMessage: string | null;
  onOutputPathChange: (nextPath: string) => void;
  onStructureOnlyChange: (nextValue: boolean) => void;
  onPickOutputPath: () => Promise<void>;
  onPreview: () => Promise<void>;
  onExport: () => Promise<void>;
};

export function ExportPanel({
  busy,
  outputPath,
  structureOnly,
  preview,
  exportResult,
  selectionSummary,
  errorMessage,
  onOutputPathChange,
  onStructureOnlyChange,
  onPickOutputPath,
  onPreview,
  onExport,
}: ExportPanelProps) {
  return (
    <section className="panel">
      <div className="panel-header">
        <div className="panel-header-icon">🚀</div>
        <h2>Preview & Export</h2>
        <span className="panel-step">Step 3</span>
      </div>
      <div className="panel-body">

        <div className="field">
          <label htmlFor="output-path">Output File</label>
          <input
            id="output-path"
            value={outputPath}
            onChange={(e) => onOutputPathChange(e.currentTarget.value)}
            placeholder="D:/exports/codebase.txt"
          />
        </div>

        <div className="actions">
          <button className="btn" onClick={() => void onPickOutputPath()} disabled={busy}>
            Browse
          </button>
          <button className="btn" onClick={() => void onPreview()} disabled={busy}>
            Preview
          </button>
          <button className="btn primary" onClick={() => void onExport()} disabled={busy}>
            Export
          </button>
        </div>

        <div className="field">
          <label htmlFor="structure-only">
            <input
              id="structure-only"
              type="checkbox"
              checked={structureOnly}
              onChange={(e) => onStructureOnlyChange(e.currentTarget.checked)}
            />
            Structure only (no file contents)
          </label>
        </div>

        {/* Stats row: Preview + Selection side by side */}
        <div className="stats-row">
          <div className={`status-card${preview ? " accent" : ""}`}>
            <h3>Preview</h3>
            {preview ? (
              <>
                <p className="stat-num">{preview.includedFiles}</p>
                <p className="stat-label">files · {formatBytes(preview.estimatedBytes)}</p>
                <p className="meta">{preview.warnings.length ? `⚠ ${preview.warnings[0]}` : "No warnings"}</p>
              </>
            ) : (
              <p className="meta">Run Preview first.</p>
            )}
          </div>

          <div className={`status-card${selectionSummary ? " accent" : ""}`}>
            <h3>Selection</h3>
            {selectionSummary ? (
              <>
                <p className="stat-num">{selectionSummary.includedFiles}</p>
                <p className="stat-label">included</p>
                <p className="meta">{selectionSummary.excludedFiles} excluded</p>
              </>
            ) : (
              <p className="meta">Run Evaluate first.</p>
            )}
          </div>
        </div>

        {/* Export Result */}
        {exportResult && (
          <div className="status-card success">
            <h3>Export Result</h3>
            <p className="stat-num">{exportResult.exportedFiles}</p>
            <p className="stat-label">files exported · {formatBytes(exportResult.totalBytesWritten)}</p>
            <p className="meta" style={{ marginTop: "0.4rem", wordBreak: "break-all" }}>
              → {exportResult.outputPath}
            </p>
            {exportResult.skippedFiles > 0 && (
              <p className="meta">⚠ {exportResult.skippedFiles} skipped</p>
            )}
          </div>
        )}

        {errorMessage && <p className="error">{errorMessage}</p>}
      </div>
    </section>
  );
}

function formatBytes(bytes: number): string {
  if (bytes < 1024) return `${bytes} B`;
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
  return `${(bytes / 1024 / 1024).toFixed(2)} MB`;
}

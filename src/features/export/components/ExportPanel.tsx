import type {
  ExportResult,
  PreviewMeta,
  SelectionSummary,
} from "../../../shared/types/export";

type ExportPanelProps = {
  busy: boolean;
  outputPath: string;
  preview: PreviewMeta | null;
  exportResult: ExportResult | null;
  selectionSummary: SelectionSummary | null;
  errorMessage: string | null;
  onOutputPathChange: (nextPath: string) => void;
  onPreview: () => Promise<void>;
  onExport: () => Promise<void>;
};

export function ExportPanel({
  busy,
  outputPath,
  preview,
  exportResult,
  selectionSummary,
  errorMessage,
  onOutputPathChange,
  onPreview,
  onExport,
}: ExportPanelProps) {
  return (
    <section className="panel">
      <div className="panel-header">
        <h2>Preview and Export</h2>
      </div>
      <div className="panel-body">
        <div className="field">
          <label htmlFor="output-path">Output File</label>
          <input
            id="output-path"
            value={outputPath}
            onChange={(event) => onOutputPathChange(event.currentTarget.value)}
            placeholder="D:/exports/codebase.txt"
          />
        </div>

        <div className="actions">
          <button className="btn" onClick={() => void onPreview()} disabled={busy}>
            Preview
          </button>
          <button className="btn primary" onClick={() => void onExport()} disabled={busy}>
            Export
          </button>
        </div>

        <div className="status-card">
          <h3>Preview Stats</h3>
          {preview ? (
            <>
              <p>Included files: {preview.includedFiles}</p>
              <p>Estimated bytes: {preview.estimatedBytes}</p>
              <p>Estimated tokens: {preview.estimatedTokens ?? "Planned in v1.1"}</p>
              <p className="meta">Warnings: {preview.warnings.join(" | ") || "None"}</p>
            </>
          ) : (
            <p className="meta">No preview yet.</p>
          )}
        </div>

        <div className="status-card">
          <h3>Export Result</h3>
          {exportResult ? (
            <>
              <p>Output path: {exportResult.outputPath}</p>
              <p>Exported files: {exportResult.exportedFiles}</p>
              <p>Skipped files: {exportResult.skippedFiles}</p>
              <p>Total bytes written: {exportResult.totalBytesWritten}</p>
              <p className="meta">Notes: {exportResult.notes.join(" | ") || "None"}</p>
            </>
          ) : (
            <p className="meta">No export yet.</p>
          )}
        </div>

        <div className="status-card">
          <h3>Current Selection</h3>
          {selectionSummary ? (
            <>
              <p>Included: {selectionSummary.includedFiles}</p>
              <p>Excluded: {selectionSummary.excludedFiles}</p>
            </>
          ) : (
            <p className="meta">No evaluation yet.</p>
          )}
        </div>

        {errorMessage ? <p className="error">{errorMessage}</p> : null}
      </div>
    </section>
  );
}

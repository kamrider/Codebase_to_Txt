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
        <h2>预览与导出</h2>
      </div>
      <div className="panel-body">
        <div className="field">
          <label htmlFor="output-path">输出文件</label>
          <input
            id="output-path"
            value={outputPath}
            onChange={(event) => onOutputPathChange(event.currentTarget.value)}
            placeholder="D:/exports/codebase.txt"
          />
        </div>

        <div className="actions">
          <button className="btn" onClick={() => void onPreview()} disabled={busy}>
            预览导出（占位）
          </button>
          <button className="btn primary" onClick={() => void onExport()} disabled={busy}>
            执行导出（占位）
          </button>
        </div>

        <div className="status-card">
          <h3>预览统计</h3>
          {preview ? (
            <>
              <p>包含文件: {preview.includedFiles}</p>
              <p>估算字节: {preview.estimatedBytes}</p>
              <p>估算 Token: {preview.estimatedTokens ?? "V1.1 规划中"}</p>
              <p className="meta">提示: {preview.warnings.join(" | ")}</p>
            </>
          ) : (
            <p className="meta">尚未预览。</p>
          )}
        </div>

        <div className="status-card">
          <h3>导出结果</h3>
          {exportResult ? (
            <>
              <p>输出路径: {exportResult.outputPath}</p>
              <p>导出文件数: {exportResult.exportedFiles}</p>
              <p>跳过文件数: {exportResult.skippedFiles}</p>
              <p>写入字节: {exportResult.totalBytesWritten}</p>
              <p className="meta">备注: {exportResult.notes.join(" | ")}</p>
            </>
          ) : (
            <p className="meta">尚未导出。</p>
          )}
        </div>

        <div className="status-card">
          <h3>当前选择摘要</h3>
          {selectionSummary ? (
            <>
              <p>包含: {selectionSummary.includedFiles}</p>
              <p>排除: {selectionSummary.excludedFiles}</p>
            </>
          ) : (
            <p className="meta">尚未评估。</p>
          )}
        </div>

        {errorMessage ? <p className="error">{errorMessage}</p> : null}
      </div>
    </section>
  );
}

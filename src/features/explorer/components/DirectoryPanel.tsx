import type { SelectionSummary, TreeNode } from "../../../shared/types/export";

type DirectoryPanelProps = {
  rootPath: string;
  busy: boolean;
  tree: TreeNode | null;
  selectionSummary: SelectionSummary | null;
  onRootPathChange: (nextPath: string) => void;
  onScan: () => Promise<void>;
  onEvaluate: () => Promise<void>;
};

export function DirectoryPanel({
  rootPath,
  busy,
  tree,
  selectionSummary,
  onRootPathChange,
  onScan,
  onEvaluate,
}: DirectoryPanelProps) {
  return (
    <section className="panel">
      <div className="panel-header">
        <h2>目录扫描与结构</h2>
      </div>
      <div className="panel-body">
        <div className="field">
          <label htmlFor="root-path">根目录路径</label>
          <input
            id="root-path"
            value={rootPath}
            onChange={(event) => onRootPathChange(event.currentTarget.value)}
            placeholder="例如 D:/github_projects"
          />
        </div>
        <div className="actions">
          <button className="btn primary" onClick={() => void onScan()} disabled={busy}>
            扫描目录（占位）
          </button>
          <button className="btn" onClick={() => void onEvaluate()} disabled={busy}>
            评估选择（占位）
          </button>
        </div>

        <div className="status-card">
          <h3>选择统计</h3>
          {selectionSummary ? (
            <>
              <p>包含文件: {selectionSummary.includedFiles}</p>
              <p>排除文件: {selectionSummary.excludedFiles}</p>
              <p className="meta">提示: {selectionSummary.warnings.join(" | ")}</p>
            </>
          ) : (
            <p className="meta">尚未评估，点击“评估选择”查看占位结果。</p>
          )}
        </div>

        <div className="tree-box">
          {tree ? (
            <ul className="tree-list">
              <TreeNodeLine node={tree} />
            </ul>
          ) : (
            <p className="meta">暂无目录树，先执行扫描。</p>
          )}
        </div>
      </div>
    </section>
  );
}

type TreeViewProps = {
  node: TreeNode;
};

function TreeNodeLine({ node }: TreeViewProps) {
  return (
    <li className="tree-item">
      [{node.isDir ? "DIR" : "FILE"}] {node.name || node.path}
      {node.children.length > 0 ? (
        <ul className="tree-list">
          {node.children.map((childNode) => (
            <TreeNodeLine key={childNode.path} node={childNode} />
          ))}
        </ul>
      ) : null}
    </li>
  );
}

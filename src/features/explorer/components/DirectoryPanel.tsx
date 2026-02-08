import type {
  ManualSelectionState,
  SelectionSummary,
  TreeNode,
} from "../../../shared/types/export";

type DirectoryPanelProps = {
  rootPath: string;
  busy: boolean;
  tree: TreeNode | null;
  selectionSummary: SelectionSummary | null;
  expandedPaths: Set<string>;
  loadingPaths: Set<string>;
  manualSelections: Record<string, ManualSelectionState>;
  onRootPathChange: (nextPath: string) => void;
  onScan: () => Promise<void>;
  onEvaluate: () => Promise<void>;
  onToggleNode: (node: TreeNode) => Promise<void>;
  onCycleManualSelection: (path: string) => void;
};

export function DirectoryPanel({
  rootPath,
  busy,
  tree,
  selectionSummary,
  expandedPaths,
  loadingPaths,
  manualSelections,
  onRootPathChange,
  onScan,
  onEvaluate,
  onToggleNode,
  onCycleManualSelection,
}: DirectoryPanelProps) {
  return (
    <section className="panel">
      <div className="panel-header">
        <h2>Directory Scan</h2>
      </div>
      <div className="panel-body">
        <div className="field">
          <label htmlFor="root-path">Root Path</label>
          <input
            id="root-path"
            value={rootPath}
            onChange={(event) => onRootPathChange(event.currentTarget.value)}
            placeholder="Example: D:/github_projects/Codebase_to_Txt"
          />
        </div>
        <div className="actions">
          <button className="btn primary" onClick={() => void onScan()} disabled={busy}>
            Scan
          </button>
          <button className="btn" onClick={() => void onEvaluate()} disabled={busy}>
            Evaluate
          </button>
        </div>

        <div className="status-card">
          <h3>Selection Summary</h3>
          {selectionSummary ? (
            <>
              <p>Included files: {selectionSummary.includedFiles}</p>
              <p>Excluded files: {selectionSummary.excludedFiles}</p>
              <p className="meta">Warnings: {selectionSummary.warnings.join(" | ") || "None"}</p>
            </>
          ) : (
            <p className="meta">No evaluation yet.</p>
          )}
        </div>

        <div className="tree-box">
          {tree ? (
            <ul className="tree-list">
              <TreeNodeLine
                node={tree}
                busy={busy}
                expandedPaths={expandedPaths}
                loadingPaths={loadingPaths}
                manualSelections={manualSelections}
                onToggleNode={onToggleNode}
                onCycleManualSelection={onCycleManualSelection}
              />
            </ul>
          ) : (
            <p className="meta">No tree data yet. Run scan first.</p>
          )}
        </div>
      </div>
    </section>
  );
}

type TreeNodeLineProps = {
  node: TreeNode;
  busy: boolean;
  expandedPaths: Set<string>;
  loadingPaths: Set<string>;
  manualSelections: Record<string, ManualSelectionState>;
  onToggleNode: (node: TreeNode) => Promise<void>;
  onCycleManualSelection: (path: string) => void;
};

function TreeNodeLine({
  node,
  busy,
  expandedPaths,
  loadingPaths,
  manualSelections,
  onToggleNode,
  onCycleManualSelection,
}: TreeNodeLineProps) {
  const isExpanded = expandedPaths.has(node.path);
  const isLoading = loadingPaths.has(node.path);
  const canExpand = node.isDir && (node.childrenCount === null || node.childrenCount > 0);
  const manualState = manualSelections[node.path] ?? "inherit";
  const manualEditable = node.path !== ".";

  return (
    <li className="tree-item">
      {canExpand ? (
        <button className="btn" onClick={() => void onToggleNode(node)} disabled={isLoading}>
          {isLoading ? "Loading..." : isExpanded ? "Collapse" : "Expand"}
        </button>
      ) : null}{" "}
      {manualEditable ? (
        <button className="btn" onClick={() => onCycleManualSelection(node.path)} disabled={busy}>
          Rule: {manualState}
        </button>
      ) : null}{" "}
      [{node.isDir ? "DIR" : "FILE"}] {node.name || node.path}
      {isExpanded && node.children.length > 0 ? (
        <ul className="tree-list">
          {node.children.map((childNode) => (
            <TreeNodeLine
              key={childNode.path}
              node={childNode}
              busy={busy}
              expandedPaths={expandedPaths}
              loadingPaths={loadingPaths}
              manualSelections={manualSelections}
              onToggleNode={onToggleNode}
              onCycleManualSelection={onCycleManualSelection}
            />
          ))}
        </ul>
      ) : null}
    </li>
  );
}
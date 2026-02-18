import { useMemo, type ReactNode } from "react";
import { Icon } from "@iconify/react";
import { Tree } from "antd";
import { getIconForFile, getIconForFolder, getIconForOpenFolder } from "vscode-icons-js";
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
  onSyncManualSelections: (
    checkedPaths: string[],
    changedPath: string,
    changedChecked: boolean,
  ) => void;
};

type UITreeNode = {
  key: string;
  isLeaf?: boolean;
  disableCheckbox?: boolean;
  title: ReactNode;
  children?: UITreeNode[];
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
  onSyncManualSelections,
}: DirectoryPanelProps) {
  const nodeLookup = useMemo(() => {
    const lookup = new Map<string, TreeNode>();
    if (!tree) {
      return lookup;
    }

    const walk = (node: TreeNode) => {
      lookup.set(node.path, node);
      for (const childNode of node.children) {
        walk(childNode);
      }
    };

    walk(tree);
    return lookup;
  }, [tree]);

  const treeData = useMemo<UITreeNode[]>(() => {
    if (!tree) {
      return [];
    }

    const toTreeDataNode = (node: TreeNode): UITreeNode => {
      const isLoading = loadingPaths.has(node.path);
      const isLeaf = !node.isDir || node.childrenCount === 0;
      const isExpanded = expandedPaths.has(node.path);
      const iconName = getNodeIconName(node, isExpanded);

      return {
        key: node.path,
        isLeaf,
        disableCheckbox: node.path === ".",
        title: (
          <div className="tree-node-line">
            <Icon icon={`vscode-icons:${iconName}`} className="tree-node-icon" />
            <span className={`tree-node-label${node.ignoredByGitignore ? " is-gitignored" : ""}`}>
              {node.name || node.path}
              {isLoading ? " (Loading...)" : ""}
            </span>
            {node.ignoredByGitignore ? <span className="tree-node-meta">gitignored</span> : null}
          </div>
        ),
        children: node.children.map(toTreeDataNode),
      };
    };

    return [toTreeDataNode(tree)];
  }, [tree, loadingPaths, expandedPaths]);

  const checkedKeys = useMemo(
    () => {
      if (!tree) {
        return [];
      }

      const keys: string[] = [];
      const walk = (node: TreeNode, inheritedOverride: ManualSelectionState | null) => {
        const ownManualState = manualSelections[node.path];
        const nextOverride =
          ownManualState === "include" || ownManualState === "exclude"
            ? ownManualState
            : inheritedOverride;
        const defaultChecked = node.path !== "." && !node.ignoredByGitignore;
        const effectiveChecked =
          nextOverride === "include"
            ? true
            : nextOverride === "exclude"
              ? false
              : defaultChecked;
        const hasLoadedChildren = node.children.length > 0;

        if (node.path !== "." && !hasLoadedChildren && effectiveChecked) {
          keys.push(node.path);
        }

        for (const childNode of node.children) {
          walk(childNode, nextOverride);
        }
      };

      walk(tree, null);
      return keys;
    },
    [tree, manualSelections],
  );

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
            <Tree
              className="directory-tree"
              treeData={treeData}
              expandedKeys={Array.from(expandedPaths)}
              checkedKeys={checkedKeys}
              selectable={false}
              checkable
              showLine
              onCheck={(nextCheckedKeys, info) => {
                const checkedPaths = Array.isArray(nextCheckedKeys)
                  ? nextCheckedKeys.map((item) => String(item))
                  : nextCheckedKeys.checked.map((item) => String(item));
                const changedPath = String(info.node.key);
                onSyncManualSelections(checkedPaths, changedPath, info.checked);
              }}
              onExpand={(_, info) => {
                const targetPath = String(info.node.key);
                const targetNode = nodeLookup.get(targetPath);
                if (targetNode) {
                  void onToggleNode(targetNode);
                }
              }}
            />
          ) : (
            <p className="meta">No tree data yet. Run scan first.</p>
          )}
        </div>
      </div>
    </section>
  );
}

const SPECIAL_FILE_ICONS: Record<string, string> = {
  ".dockerignore": "file_type_docker.svg",
  ".ignore": "file_type_git.svg",
};

const ICON_FALLBACKS: Record<string, string> = {
  "file-type-makefile": "file-type-config",
  "file-type-pdf": "file-type-pdf2",
};

function getNodeIconName(node: TreeNode, isExpanded: boolean): string {
  if (node.isDir) {
    const folderName = (node.name || node.path || "").toLowerCase();
    const iconFileName = isExpanded ? getIconForOpenFolder(folderName) : getIconForFolder(folderName);
    return toVscodeIconName(iconFileName);
  }

  const fileName = (node.name || node.path || "").toLowerCase();
  const iconFileName = SPECIAL_FILE_ICONS[fileName] ?? getIconForFile(fileName);
  return toVscodeIconName(iconFileName);
}

function toVscodeIconName(iconFileName: string): string {
  const normalized = iconFileName.replace(/\.svg$/i, "").replace(/_/g, "-");
  return ICON_FALLBACKS[normalized] ?? normalized;
}

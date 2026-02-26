export type LargeFileStrategy = "truncate" | "skip";
export type OutputFormat = "txt" | "md";
export type ManualSelectionState = "include" | "exclude" | "inherit";

export interface ExportConfig {
  rootPath: string;
  useGitignore: boolean;
  includeGlobs: string[];
  excludeGlobs: string[];
  includeExtensions: string[];
  excludeExtensions: string[];
  structureOnly: boolean;
  maxFileSizeKB: number;
  largeFileStrategy: LargeFileStrategy;
  manualSelections: Record<string, ManualSelectionState>;
  outputFormat: OutputFormat;
}

export type RulesDraft = Pick<
  ExportConfig,
  "useGitignore" | "includeGlobs" | "excludeGlobs" | "includeExtensions" | "excludeExtensions"
>;

export interface TreeNode {
  path: string;
  name: string;
  isDir: boolean;
  childrenCount: number | null;
  includedByRules: boolean;
  ignoredByGitignore: boolean;
  children: TreeNode[];
}

export interface SelectionSummary {
  includedFiles: number;
  excludedFiles: number;
  warnings: string[];
}

export interface PreviewMeta {
  includedFiles: number;
  estimatedBytes: number;
  estimatedTokens: number | null;
  warnings: string[];
}

export interface ExportResult {
  outputPath: string;
  exportedFiles: number;
  skippedFiles: number;
  totalBytesWritten: number;
  notes: string[];
}

export const defaultExportConfig: ExportConfig = {
  rootPath: "",
  useGitignore: true,
  includeGlobs: [],
  excludeGlobs: [],
  includeExtensions: [],
  excludeExtensions: [],
  structureOnly: false,
  maxFileSizeKB: 256,
  largeFileStrategy: "truncate",
  manualSelections: {},
  outputFormat: "txt",
};

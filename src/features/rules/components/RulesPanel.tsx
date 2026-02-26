import { useEffect, useState } from "react";
import type {
  ExportConfig,
  LargeFileStrategy,
  OutputFormat,
  RulesDraft,
} from "../../../shared/types/export";

type RulesPanelProps = {
  config: ExportConfig;
  rulesDraft: RulesDraft;
  rulesDirty: boolean;
  busy: boolean;
  onUpdateConfig: (patch: Partial<ExportConfig>) => void;
  onUpdateRulesDraft: (patch: Partial<RulesDraft>) => void;
  onApplyRules: () => Promise<ExportConfig | null>;
};

type ListField =
  | "includeGlobs"
  | "excludeGlobs"
  | "includeExtensions"
  | "excludeExtensions";

function parseCsv(rawValue: string): string[] {
  return rawValue
    .split(/[\u002C\uFF0C]/)
    .map((item) => item.trim())
    .filter((item) => item.length > 0);
}

export function RulesPanel({
  config,
  rulesDraft,
  rulesDirty,
  busy,
  onUpdateConfig,
  onUpdateRulesDraft,
  onApplyRules,
}: RulesPanelProps) {
  const [includeGlobsText, setIncludeGlobsText] = useState(() => rulesDraft.includeGlobs.join(", "));
  const [excludeGlobsText, setExcludeGlobsText] = useState(() => rulesDraft.excludeGlobs.join(", "));
  const [includeExtensionsText, setIncludeExtensionsText] = useState(() =>
    rulesDraft.includeExtensions.join(", "),
  );
  const [excludeExtensionsText, setExcludeExtensionsText] = useState(() =>
    rulesDraft.excludeExtensions.join(", "),
  );

  useEffect(() => {
    setIncludeGlobsText(rulesDraft.includeGlobs.join(", "));
    setExcludeGlobsText(rulesDraft.excludeGlobs.join(", "));
    setIncludeExtensionsText(rulesDraft.includeExtensions.join(", "));
    setExcludeExtensionsText(rulesDraft.excludeExtensions.join(", "));
  }, [rulesDraft]);

  const updateListField = (field: ListField, rawValue: string) => {
    const nextList = parseCsv(rawValue);
    if (field === "includeGlobs") {
      onUpdateRulesDraft({ includeGlobs: nextList });
      return;
    }
    if (field === "excludeGlobs") {
      onUpdateRulesDraft({ excludeGlobs: nextList });
      return;
    }
    if (field === "includeExtensions") {
      onUpdateRulesDraft({ includeExtensions: nextList });
      return;
    }
    onUpdateRulesDraft({ excludeExtensions: nextList });
  };

  return (
    <section className="panel">
      <div className="panel-header">
        <h2>Rules</h2>
      </div>
      <div className="panel-body">
        <div className="field">
          <label htmlFor="use-gitignore">
            <input
              id="use-gitignore"
              type="checkbox"
              checked={rulesDraft.useGitignore}
              onChange={(event) => onUpdateRulesDraft({ useGitignore: event.currentTarget.checked })}
            />{" "}
            Apply .gitignore
          </label>
        </div>

        <div className="field">
          <label htmlFor="include-globs">Include Globs (comma-separated)</label>
          <input
            id="include-globs"
            value={includeGlobsText}
            onChange={(event) => {
              const raw = event.currentTarget.value;
              setIncludeGlobsText(raw);
              updateListField("includeGlobs", raw);
            }}
            placeholder="src/**, docs/**"
          />
        </div>

        <div className="field">
          <label htmlFor="exclude-globs">Exclude Globs (comma-separated)</label>
          <input
            id="exclude-globs"
            value={excludeGlobsText}
            onChange={(event) => {
              const raw = event.currentTarget.value;
              setExcludeGlobsText(raw);
              updateListField("excludeGlobs", raw);
            }}
            placeholder="node_modules/**, dist/**"
          />
        </div>

        <div className="field">
          <label htmlFor="include-extensions">Include Extensions (comma-separated)</label>
          <input
            id="include-extensions"
            value={includeExtensionsText}
            onChange={(event) => {
              const raw = event.currentTarget.value;
              setIncludeExtensionsText(raw);
              updateListField("includeExtensions", raw);
            }}
            placeholder=".ts, .tsx, .md"
          />
        </div>

        <div className="field">
          <label htmlFor="exclude-extensions">Exclude Extensions (comma-separated)</label>
          <input
            id="exclude-extensions"
            value={excludeExtensionsText}
            onChange={(event) => {
              const raw = event.currentTarget.value;
              setExcludeExtensionsText(raw);
              updateListField("excludeExtensions", raw);
            }}
            placeholder=".png, .ico, .zip"
          />
        </div>

        <div className="actions">
          <button
            className="btn primary"
            onClick={() => void onApplyRules()}
            disabled={busy || !rulesDirty}
          >
            Apply Rules
          </button>
          {rulesDirty ? <p className="meta">Pending rule changes.</p> : null}
        </div>

        <div className="field">
          <label htmlFor="max-size-kb">Max File Size (KB)</label>
          <input
            id="max-size-kb"
            type="number"
            min={1}
            value={config.maxFileSizeKB}
            onChange={(event) =>
              onUpdateConfig({ maxFileSizeKB: Number(event.currentTarget.value) || 1 })
            }
          />
        </div>

        <div className="field">
          <label htmlFor="large-file-strategy">Large File Strategy</label>
          <select
            id="large-file-strategy"
            value={config.largeFileStrategy}
            onChange={(event) =>
              onUpdateConfig({
                largeFileStrategy: event.currentTarget.value as LargeFileStrategy,
              })
            }
          >
            <option value="truncate">truncate</option>
            <option value="skip">skip</option>
          </select>
        </div>

        <div className="field">
          <label htmlFor="output-format">Output Format</label>
          <select
            id="output-format"
            value={config.outputFormat}
            onChange={(event) =>
              onUpdateConfig({ outputFormat: event.currentTarget.value as OutputFormat })
            }
          >
            <option value="txt">txt</option>
            <option value="md">md</option>
          </select>
        </div>
      </div>
    </section>
  );
}

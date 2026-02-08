import type {
  ExportConfig,
  LargeFileStrategy,
  OutputFormat,
} from "../../../shared/types/export";

type RulesPanelProps = {
  config: ExportConfig;
  onUpdateConfig: (patch: Partial<ExportConfig>) => void;
};

type ListField =
  | "includeGlobs"
  | "excludeGlobs"
  | "includeExtensions"
  | "excludeExtensions";

function parseCsv(rawValue: string): string[] {
  return rawValue
    .split(",")
    .map((item) => item.trim())
    .filter((item) => item.length > 0);
}

export function RulesPanel({ config, onUpdateConfig }: RulesPanelProps) {
  const updateListField = (field: ListField, rawValue: string) => {
    const nextList = parseCsv(rawValue);
    if (field === "includeGlobs") {
      onUpdateConfig({ includeGlobs: nextList });
      return;
    }
    if (field === "excludeGlobs") {
      onUpdateConfig({ excludeGlobs: nextList });
      return;
    }
    if (field === "includeExtensions") {
      onUpdateConfig({ includeExtensions: nextList });
      return;
    }
    onUpdateConfig({ excludeExtensions: nextList });
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
              checked={config.useGitignore}
              onChange={(event) => onUpdateConfig({ useGitignore: event.currentTarget.checked })}
            />{" "}
            Apply .gitignore
          </label>
        </div>

        <div className="field">
          <label htmlFor="include-globs">Include Globs (comma-separated)</label>
          <input
            id="include-globs"
            value={config.includeGlobs.join(", ")}
            onChange={(event) => updateListField("includeGlobs", event.currentTarget.value)}
            placeholder="src/**, docs/**"
          />
        </div>

        <div className="field">
          <label htmlFor="exclude-globs">Exclude Globs (comma-separated)</label>
          <input
            id="exclude-globs"
            value={config.excludeGlobs.join(", ")}
            onChange={(event) => updateListField("excludeGlobs", event.currentTarget.value)}
            placeholder="node_modules/**, dist/**"
          />
        </div>

        <div className="field">
          <label htmlFor="include-extensions">Include Extensions (comma-separated)</label>
          <input
            id="include-extensions"
            value={config.includeExtensions.join(", ")}
            onChange={(event) =>
              updateListField("includeExtensions", event.currentTarget.value)
            }
            placeholder=".ts, .tsx, .md"
          />
        </div>

        <div className="field">
          <label htmlFor="exclude-extensions">Exclude Extensions (comma-separated)</label>
          <input
            id="exclude-extensions"
            value={config.excludeExtensions.join(", ")}
            onChange={(event) =>
              updateListField("excludeExtensions", event.currentTarget.value)
            }
            placeholder=".png, .ico, .zip"
          />
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

import { WorkbenchPage } from "../pages/workbench/WorkbenchPage";
import { ThemeSwitcher } from "../features/theme/ThemeSwitcher";
import { useTheme } from "../shared/hooks/useTheme";

export function AppShell() {
  const { theme, setTheme } = useTheme();

  return (
    <div className="app-shell">
      <header className="app-header">
        <div className="app-brand">
          <div className="app-brand-icon">⚡</div>
          <div>
            <p className="kicker">Codebase to TXT</p>
            <h1>Workspace Builder</h1>
          </div>
        </div>
        <div className="app-header-right">
          <ThemeSwitcher current={theme} onChange={setTheme} />
          <span className="badge">v1.0.3</span>
        </div>
      </header>
      <WorkbenchPage />
    </div>
  );
}

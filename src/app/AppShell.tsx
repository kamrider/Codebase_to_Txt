import { WorkbenchPage } from "../pages/workbench/WorkbenchPage";

export function AppShell() {
  return (
    <div className="app-shell">
      <header className="app-header">
        <div>
          <p className="kicker">Codebase to TXT</p>
          <h1>Workspace Builder</h1>
        </div>
        <span className="badge">Skeleton v0.1</span>
      </header>
      <WorkbenchPage />
    </div>
  );
}

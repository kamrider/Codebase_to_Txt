import type { ThemeId } from "../../shared/hooks/useTheme";

type ThemeDef = { id: ThemeId; label: string; color: string };

const NIGHT_THEMES: ThemeDef[] = [
  { id: "purple", label: "Purple Dusk",  color: "#7c3aed" },
  { id: "ocean",  label: "Ocean Night",  color: "#2563eb" },
  { id: "forest", label: "Forest Night", color: "#059669" },
];

const DAY_THEMES: ThemeDef[] = [
  { id: "pearl",  label: "Pearl Day",   color: "#7c3aed" },
  { id: "sky",    label: "Sky Day",     color: "#2563eb" },
  { id: "meadow", label: "Meadow Day",  color: "#16a34a" },
];

type Props = { current: ThemeId; onChange: (id: ThemeId) => void };

export function ThemeSwitcher({ current, onChange }: Props) {
  return (
    <div className="theme-switcher">
      <span className="theme-group-label">🌙</span>
      {NIGHT_THEMES.map((t) => (
        <button
          key={t.id}
          className={`theme-dot${current === t.id ? " active" : ""}`}
          style={{ "--dot-color": t.color } as React.CSSProperties}
          title={t.label}
          onClick={() => onChange(t.id)}
        />
      ))}
      <span className="theme-divider" />
      <span className="theme-group-label">☀️</span>
      {DAY_THEMES.map((t) => (
        <button
          key={t.id}
          className={`theme-dot${current === t.id ? " active" : ""}`}
          style={{ "--dot-color": t.color } as React.CSSProperties}
          title={t.label}
          onClick={() => onChange(t.id)}
        />
      ))}
    </div>
  );
}

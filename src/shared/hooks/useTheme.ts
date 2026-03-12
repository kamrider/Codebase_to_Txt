import { useEffect, useState } from "react";

export type ThemeId = "purple" | "ocean" | "forest" | "pearl" | "sky" | "meadow";

const STORAGE_KEY = "codebase_to_txt:theme";

export function useTheme() {
  const [theme, setTheme] = useState<ThemeId>(() => {
    try {
      return (localStorage.getItem(STORAGE_KEY) as ThemeId) ?? "pearl";
    } catch {
      return "pearl";
    }
  });

  useEffect(() => {
    document.documentElement.setAttribute("data-theme", theme);
    try {
      localStorage.setItem(STORAGE_KEY, theme);
    } catch {
      // ignore
    }
  }, [theme]);

  return { theme, setTheme };
}

/**
 * Theme Management
 * Handles dark/light mode with system preference detection
 */

import {
  createContext,
  createSignal,
  createEffect,
  onMount,
  useContext,
  type ParentComponent,
  type Accessor,
} from "solid-js";

export type Theme = "light" | "dark" | "system";

interface ThemeContextValue {
  theme: Accessor<Theme>;
  resolvedTheme: () => "light" | "dark";
  setTheme: (theme: Theme) => void;
}

const THEME_KEY = "godstack-theme";

function getSystemTheme(): "light" | "dark" {
  if (typeof window === "undefined") return "light";
  return window.matchMedia("(prefers-color-scheme: dark)").matches
    ? "dark"
    : "light";
}

function getStoredTheme(): Theme {
  if (typeof window === "undefined") return "system";
  const stored = localStorage.getItem(THEME_KEY);
  if (stored === "light" || stored === "dark" || stored === "system") {
    return stored;
  }
  return "system";
}

const ThemeContext = createContext<ThemeContextValue>();

export const ThemeProvider: ParentComponent = (props) => {
  const [theme, setThemeState] = createSignal<Theme>(getStoredTheme());
  const [systemTheme, setSystemTheme] = createSignal<"light" | "dark">(
    getSystemTheme()
  );

  // Resolved theme (actual light/dark value)
  const resolvedTheme = (): "light" | "dark" =>
    theme() === "system" ? systemTheme() : theme() as "light" | "dark";

  // Apply theme to document
  createEffect(() => {
    const resolved = resolvedTheme();
    const root = document.documentElement;

    if (resolved === "dark") {
      root.classList.add("dark");
    } else {
      root.classList.remove("dark");
    }
  });

  // Listen for system theme changes
  onMount(() => {
    const mediaQuery = window.matchMedia("(prefers-color-scheme: dark)");

    const handleChange = (e: MediaQueryListEvent) => {
      setSystemTheme(e.matches ? "dark" : "light");
    };

    mediaQuery.addEventListener("change", handleChange);

    return () => {
      mediaQuery.removeEventListener("change", handleChange);
    };
  });

  const setTheme = (newTheme: Theme) => {
    setThemeState(newTheme);
    localStorage.setItem(THEME_KEY, newTheme);
  };

  return (
    <ThemeContext.Provider value={{ theme, resolvedTheme, setTheme }}>
      {props.children}
    </ThemeContext.Provider>
  );
};

export function useTheme(): ThemeContextValue {
  const context = useContext(ThemeContext);
  if (!context) {
    throw new Error("useTheme must be used within a ThemeProvider");
  }
  return context;
}

/**
 * Theme Toggle Component
 * Allows users to switch between light, dark, and system themes
 */

import { Show } from "solid-js";
import { useTheme, type Theme } from "../lib/theme";
import { Button } from "./ui";

// Icons
const SunIcon = () => (
  <svg
    xmlns="http://www.w3.org/2000/svg"
    width="18"
    height="18"
    viewBox="0 0 24 24"
    fill="none"
    stroke="currentColor"
    stroke-width="2"
    stroke-linecap="round"
    stroke-linejoin="round"
  >
    <circle cx="12" cy="12" r="4" />
    <path d="M12 2v2" />
    <path d="M12 20v2" />
    <path d="m4.93 4.93 1.41 1.41" />
    <path d="m17.66 17.66 1.41 1.41" />
    <path d="M2 12h2" />
    <path d="M20 12h2" />
    <path d="m6.34 17.66-1.41 1.41" />
    <path d="m19.07 4.93-1.41 1.41" />
  </svg>
);

const MoonIcon = () => (
  <svg
    xmlns="http://www.w3.org/2000/svg"
    width="18"
    height="18"
    viewBox="0 0 24 24"
    fill="none"
    stroke="currentColor"
    stroke-width="2"
    stroke-linecap="round"
    stroke-linejoin="round"
  >
    <path d="M12 3a6 6 0 0 0 9 9 9 9 0 1 1-9-9Z" />
  </svg>
);

const MonitorIcon = () => (
  <svg
    xmlns="http://www.w3.org/2000/svg"
    width="18"
    height="18"
    viewBox="0 0 24 24"
    fill="none"
    stroke="currentColor"
    stroke-width="2"
    stroke-linecap="round"
    stroke-linejoin="round"
  >
    <rect width="20" height="14" x="2" y="3" rx="2" />
    <line x1="8" x2="16" y1="21" y2="21" />
    <line x1="12" x2="12" y1="17" y2="21" />
  </svg>
);

interface ThemeToggleProps {
  showLabel?: boolean;
}

export function ThemeToggle(props: ThemeToggleProps) {
  const { theme, setTheme } = useTheme();

  const cycleTheme = () => {
    const themes: Theme[] = ["light", "dark", "system"];
    const currentIndex = themes.indexOf(theme());
    const nextIndex = (currentIndex + 1) % themes.length;
    setTheme(themes[nextIndex]);
  };

  const getIcon = () => {
    const current = theme();
    if (current === "light") return <SunIcon />;
    if (current === "dark") return <MoonIcon />;
    return <MonitorIcon />;
  };

  const getLabel = () => {
    const current = theme();
    if (current === "light") return "Light";
    if (current === "dark") return "Dark";
    return "System";
  };

  return (
    <Button
      variant="ghost"
      size="sm"
      onClick={cycleTheme}
      class="gap-2"
      title={`Current theme: ${getLabel()}. Click to change.`}
    >
      {getIcon()}
      <Show when={props.showLabel}>
        <span class="text-sm">{getLabel()}</span>
      </Show>
    </Button>
  );
}

/**
 * ThemeSelect - Dropdown-style theme selector
 */
export function ThemeSelect() {
  const { theme, setTheme } = useTheme();

  return (
    <div class="flex items-center gap-1 rounded-lg border bg-muted p-1">
      <button
        onClick={() => setTheme("light")}
        class={`rounded-md p-2 transition-colors ${
          theme() === "light"
            ? "bg-background text-foreground shadow-sm"
            : "text-muted-foreground hover:text-foreground"
        }`}
        title="Light mode"
      >
        <SunIcon />
      </button>
      <button
        onClick={() => setTheme("dark")}
        class={`rounded-md p-2 transition-colors ${
          theme() === "dark"
            ? "bg-background text-foreground shadow-sm"
            : "text-muted-foreground hover:text-foreground"
        }`}
        title="Dark mode"
      >
        <MoonIcon />
      </button>
      <button
        onClick={() => setTheme("system")}
        class={`rounded-md p-2 transition-colors ${
          theme() === "system"
            ? "bg-background text-foreground shadow-sm"
            : "text-muted-foreground hover:text-foreground"
        }`}
        title="System preference"
      >
        <MonitorIcon />
      </button>
    </div>
  );
}

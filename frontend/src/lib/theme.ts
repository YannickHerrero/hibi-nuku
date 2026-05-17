export const THEMES = ["paper", "stone", "sage", "clay", "ink"] as const;
export type Theme = (typeof THEMES)[number];

const KEY = "nuku:theme";

export function getTheme(): Theme {
  const v = localStorage.getItem(KEY) as Theme | null;
  return v && (THEMES as readonly string[]).includes(v) ? v : "paper";
}

export function setTheme(t: Theme) {
  localStorage.setItem(KEY, t);
  document.documentElement.setAttribute("data-theme", t);
}

export function applyStoredTheme() {
  document.documentElement.setAttribute("data-theme", getTheme());
}

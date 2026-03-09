import { useState } from "react";
import type { AppSettings } from "../components/SettingsView";

const SETTINGS_STORAGE_KEY = "codex-switcher-settings";

export const defaultSettings: AppSettings = {
  locale: "en",
  sortMode: "rating",
  reloadAfterSwitch: false,
  primaryReloadTarget: "codex",
};

function loadSettings(): AppSettings {
  try {
    const saved = localStorage.getItem(SETTINGS_STORAGE_KEY);
    if (saved) {
      return { ...defaultSettings, ...JSON.parse(saved) };
    }
  } catch {}

  return defaultSettings;
}

export function useAppSettings() {
  const [settings, setSettings] = useState(loadSettings);

  function updateSettings(patch: Partial<AppSettings>) {
    setSettings((prev) => {
      const next = { ...prev, ...patch };
      localStorage.setItem(SETTINGS_STORAGE_KEY, JSON.stringify(next));
      return next;
    });
  }

  return {
    settings,
    updateSettings,
  };
}

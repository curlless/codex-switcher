import { useEffect } from "react";
import { LogicalSize, getCurrentWindow } from "@tauri-apps/api/window";

const WINDOW_STATE_KEY = "codex-switcher-desktop-window-state";
const WINDOW_STATE_SAVE_DELAY_MS = 150;

interface SavedWindowState {
  width: number;
  height: number;
}

function isTauriWindowAvailable(): boolean {
  return typeof window !== "undefined" && "__TAURI_INTERNALS__" in window;
}

function loadSavedWindowState(): SavedWindowState | null {
  try {
    const saved = localStorage.getItem(WINDOW_STATE_KEY);
    if (!saved) {
      return null;
    }

    const parsed = JSON.parse(saved) as Partial<SavedWindowState>;
    if (
      typeof parsed.width === "number" &&
      Number.isFinite(parsed.width) &&
      typeof parsed.height === "number" &&
      Number.isFinite(parsed.height)
    ) {
      return {
        width: Math.max(800, parsed.width),
        height: Math.max(600, parsed.height),
      };
    }
  } catch {}

  return null;
}

function saveWindowState(state: SavedWindowState) {
  localStorage.setItem(WINDOW_STATE_KEY, JSON.stringify(state));
}

export function useDesktopWindowState() {
  useEffect(() => {
    if (!isTauriWindowAvailable()) {
      return;
    }

    let disposed = false;
    let unlistenResize: (() => void) | null = null;
    let saveTimer: number | null = null;
    const currentWindow = getCurrentWindow();

    async function restoreAndTrackWindow() {
      const saved = loadSavedWindowState();
      if (saved) {
        await currentWindow.setSize(new LogicalSize(saved.width, saved.height));
      }

      const scaleFactor = await currentWindow.scaleFactor();
      const currentSize = (await currentWindow.innerSize()).toLogical(scaleFactor);
      saveWindowState({ width: currentSize.width, height: currentSize.height });

      unlistenResize = await currentWindow.onResized(async ({ payload }) => {
        if (disposed || (await currentWindow.isMaximized())) {
          return;
        }

        const logicalSize = payload.toLogical(await currentWindow.scaleFactor());
        if (saveTimer !== null) {
          window.clearTimeout(saveTimer);
        }
        saveTimer = window.setTimeout(() => {
          saveWindowState({
            width: logicalSize.width,
            height: logicalSize.height,
          });
        }, WINDOW_STATE_SAVE_DELAY_MS);
      });
    }

    void restoreAndTrackWindow();

    return () => {
      disposed = true;
      if (saveTimer !== null) {
        window.clearTimeout(saveTimer);
      }
      unlistenResize?.();
    };
  }, []);
}

import { useEffect } from "react";
import { LogicalSize, getCurrentWindow } from "@tauri-apps/api/window";

const WINDOW_STATE_KEY = "codex-switcher-desktop-window-state";
const WINDOW_STATE_SAVE_DELAY_MS = 150;

interface SavedWindowState {
  width: number;
  height: number;
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

function resolveCurrentWindow() {
  if (typeof window === "undefined") {
    return null;
  }

  try {
    return getCurrentWindow();
  } catch {
    return null;
  }
}

async function readCurrentWindowState(
  currentWindow: ReturnType<typeof getCurrentWindow>,
): Promise<SavedWindowState | null> {
  try {
    const scaleFactor = await currentWindow.scaleFactor();
    const currentSize = (await currentWindow.outerSize()).toLogical(scaleFactor);

    return {
      width: Math.max(800, Math.round(currentSize.width)),
      height: Math.max(600, Math.round(currentSize.height)),
    };
  } catch {
    return null;
  }
}

export function useDesktopWindowState() {
  useEffect(() => {
    const resolvedWindow = resolveCurrentWindow();
    if (!resolvedWindow) {
      return;
    }
    const currentWindow = resolvedWindow;

    let disposed = false;
    let unlistenResize: (() => void) | null = null;
    let unlistenClose: (() => void) | null = null;
    let saveTimer: number | null = null;

    function clearSaveTimer() {
      if (saveTimer !== null) {
        window.clearTimeout(saveTimer);
        saveTimer = null;
      }
    }

    async function flushWindowState() {
      if (disposed || (await currentWindow.isMaximized())) {
        return;
      }

      const state = await readCurrentWindowState(currentWindow);
      if (state) {
        saveWindowState(state);
      }
    }

    async function scheduleWindowStateSave() {
      const state = await readCurrentWindowState(currentWindow);
      if (!state || disposed) {
        return;
      }

      clearSaveTimer();
      saveTimer = window.setTimeout(() => {
        saveWindowState(state);
      }, WINDOW_STATE_SAVE_DELAY_MS);
    }

    async function restoreAndTrackWindow() {
      const saved = loadSavedWindowState();
      if (saved) {
        if (await currentWindow.isMaximized()) {
          await currentWindow.unmaximize();
        }
        await currentWindow.setSize(new LogicalSize(saved.width, saved.height));
      } else {
        await flushWindowState();
      }

      unlistenResize = await currentWindow.onResized(async () => {
        if (disposed) {
          return;
        }

        await scheduleWindowStateSave();
      });

      unlistenClose = await currentWindow.onCloseRequested(async () => {
        clearSaveTimer();
        await flushWindowState();
      });
    }

    void restoreAndTrackWindow();

    return () => {
      disposed = true;
      clearSaveTimer();
      unlistenResize?.();
      unlistenClose?.();
    };
  }, []);
}

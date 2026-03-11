import { useLayoutEffect } from "react";
import { PhysicalSize, getCurrentWindow } from "@tauri-apps/api/window";

const WINDOW_STATE_STORAGE_KEY = "codex-switcher-window-size";
const MIN_WINDOW_WIDTH = 960;
const MIN_WINDOW_HEIGHT = 640;

interface PersistedWindowSize {
  width: number;
  height: number;
}

function isTauriAvailable(): boolean {
  return typeof window !== "undefined" && "__TAURI_INTERNALS__" in window;
}

function sanitizeWindowSize(value: Partial<PersistedWindowSize> | null): PersistedWindowSize | null {
  if (!value) {
    return null;
  }

  const width = typeof value.width === "number" ? Math.round(value.width) : 0;
  const height = typeof value.height === "number" ? Math.round(value.height) : 0;

  if (width < MIN_WINDOW_WIDTH || height < MIN_WINDOW_HEIGHT) {
    return null;
  }

  return { width, height };
}

function loadWindowSize(): PersistedWindowSize | null {
  try {
    const raw = localStorage.getItem(WINDOW_STATE_STORAGE_KEY);
    if (!raw) {
      return null;
    }

    return sanitizeWindowSize(JSON.parse(raw) as Partial<PersistedWindowSize>);
  } catch {
    return null;
  }
}

function saveWindowSize(size: PersistedWindowSize) {
  try {
    localStorage.setItem(WINDOW_STATE_STORAGE_KEY, JSON.stringify(size));
  } catch {
    // Ignore storage failures. The runtime can still keep the current session alive.
  }
}

export function useDesktopWindowState() {
  useLayoutEffect(() => {
    if (!isTauriAvailable()) {
      return;
    }

    let unlistenResize: (() => void) | null = null;
    let unlistenClose: (() => void) | null = null;

    async function persistCurrentSize() {
      const currentWindow = getCurrentWindow();
      const size = await currentWindow.innerSize();
      const next = sanitizeWindowSize({
        width: size.width,
        height: size.height,
      });

      if (next) {
        saveWindowSize(next);
      }
    }

    async function attach() {
      const currentWindow = getCurrentWindow();
      const savedSize = loadWindowSize();

      if (savedSize) {
        await currentWindow.setSize(new PhysicalSize(savedSize.width, savedSize.height));
      }

      await persistCurrentSize();

      unlistenResize = await currentWindow.onResized(({ payload }) => {
        const next = sanitizeWindowSize({
          width: payload.width,
          height: payload.height,
        });

        if (next) {
          saveWindowSize(next);
        }
      });

      unlistenClose = await currentWindow.onCloseRequested(async () => {
        await persistCurrentSize();
      });
    }

    void attach();

    return () => {
      if (unlistenResize) {
        unlistenResize();
      }
      if (unlistenClose) {
        unlistenClose();
      }
    };
  }, []);
}

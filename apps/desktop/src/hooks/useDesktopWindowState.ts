import { useEffect } from "react";

export function useDesktopWindowState() {
  useEffect(() => {
    // Window-state persistence is owned by the Tauri runtime so restore happens
    // before the React tree mounts and without webview/localStorage races.
  }, []);
}

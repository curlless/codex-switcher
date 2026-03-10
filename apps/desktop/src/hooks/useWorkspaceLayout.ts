import { useCallback, useEffect, useState } from "react";

const WORKSPACE_LAYOUT_KEY = "codex-switcher-workspace-layout";
const DEFAULT_RATIO = 0.33;
const MIN_PROFILE_PANE_WIDTH = 280;
const MAX_PROFILE_PANE_WIDTH = 720;

interface WorkspaceLayout {
  profilePaneWidth: number;
}

function clampProfilePaneWidth(width: number, viewportWidth?: number): number {
  const fallbackViewport =
    typeof window !== "undefined" ? window.innerWidth : MAX_PROFILE_PANE_WIDTH;
  const currentViewport = viewportWidth ?? fallbackViewport;
  const adaptiveMax = Math.max(
    MIN_PROFILE_PANE_WIDTH,
    Math.min(MAX_PROFILE_PANE_WIDTH, Math.floor(currentViewport * 0.6)),
  );
  return Math.max(MIN_PROFILE_PANE_WIDTH, Math.min(width, adaptiveMax));
}

function defaultProfilePaneWidth(): number {
  if (typeof window === "undefined") {
    return 360;
  }

  return clampProfilePaneWidth(Math.round(window.innerWidth * DEFAULT_RATIO));
}

function loadWorkspaceLayout(): WorkspaceLayout {
  try {
    const saved = localStorage.getItem(WORKSPACE_LAYOUT_KEY);
    if (saved) {
      const parsed = JSON.parse(saved) as Partial<WorkspaceLayout>;
      if (typeof parsed.profilePaneWidth === "number") {
        return {
          profilePaneWidth: clampProfilePaneWidth(parsed.profilePaneWidth),
        };
      }
    }
  } catch {}

  return {
    profilePaneWidth: defaultProfilePaneWidth(),
  };
}

export function useWorkspaceLayout() {
  const [layout, setLayout] = useState(loadWorkspaceLayout);

  useEffect(() => {
    function handleWindowResize() {
      setLayout((prev) => {
        const next = {
          profilePaneWidth: clampProfilePaneWidth(prev.profilePaneWidth),
        };
        localStorage.setItem(WORKSPACE_LAYOUT_KEY, JSON.stringify(next));
        return next;
      });
    }

    window.addEventListener("resize", handleWindowResize);
    return () => window.removeEventListener("resize", handleWindowResize);
  }, []);

  const updateProfilePaneWidth = useCallback((width: number) => {
    setLayout((prev) => {
      const next = {
        ...prev,
        profilePaneWidth: clampProfilePaneWidth(width),
      };
      localStorage.setItem(WORKSPACE_LAYOUT_KEY, JSON.stringify(next));
      return next;
    });
  }, []);

  return {
    profilePaneWidth: layout.profilePaneWidth,
    updateProfilePaneWidth,
  };
}

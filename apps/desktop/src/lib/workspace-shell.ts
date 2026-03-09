import {
  loadActiveProfileStatus,
  loadProfilesOverview,
  loadReloadTargets,
} from "../bridge";
import type {
  ActiveProfileStatusPayload,
  DesktopCommandError,
  ProfilesOverviewPayload,
  ReloadTargetInfo,
  ReloadTargetsPayload,
} from "./contracts";

export type AppPhase = "loading" | "error" | "ready";

export interface WorkspaceShellState {
  overview: ProfilesOverviewPayload | null;
  activeStatus: ActiveProfileStatusPayload | null;
  reloadTargets: ReloadTargetsPayload | null;
  selectedLabel: string;
}

export interface WorkspaceShellSnapshot extends WorkspaceShellState {
  commandError: DesktopCommandError | null;
  phase: AppPhase;
  ok: boolean;
}

export async function bootstrapWorkspaceShell(
  current: WorkspaceShellState,
): Promise<WorkspaceShellSnapshot> {
  const [overviewResult, activeStatusResult, reloadTargetsResult] = await Promise.all([
    loadProfilesOverview(),
    loadActiveProfileStatus(),
    loadReloadTargets(),
  ]);

  let overview = current.overview;
  let activeStatus = current.activeStatus;
  let reloadTargets = current.reloadTargets;
  let selectedLabel = current.selectedLabel;
  let lastError: DesktopCommandError | null = null;

  if (overviewResult.ok) {
    overview = overviewResult.data;
  } else {
    lastError = overviewResult.error;
  }

  if (activeStatusResult.ok) {
    activeStatus = activeStatusResult.data;
    selectedLabel = activeStatusResult.data.activeProfile;
  } else {
    lastError = activeStatusResult.error;
  }

  if (reloadTargetsResult.ok) {
    reloadTargets = reloadTargetsResult.data;
  } else {
    lastError = reloadTargetsResult.error;
  }

  return {
    overview,
    activeStatus,
    reloadTargets,
    selectedLabel,
    commandError: lastError,
    phase: overview || activeStatus ? "ready" : "error",
    ok: lastError === null,
  };
}

export function selectReloadTargets(
  reloadTargets: ReloadTargetsPayload | null,
  primaryReloadTarget: "codex" | "cursor" | "all",
): ReloadTargetInfo[] {
  if (!reloadTargets?.targets.length) {
    return [];
  }

  if (primaryReloadTarget === "all") {
    return reloadTargets.targets;
  }

  return reloadTargets.targets.filter(
    (target) => target.id === primaryReloadTarget,
  );
}

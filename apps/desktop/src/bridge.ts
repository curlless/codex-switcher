import { invoke } from "@tauri-apps/api/core";
import type {
  ActiveProfileStatusPayload,
  DesktopCommandResult,
  ProfilesOverviewPayload,
  ReloadOutcomePayload,
  ReloadTarget,
  ReloadTargetsPayload,
  SwitchExecutePayload,
  SwitchPreviewPayload
} from "./lib/contracts";
import {
  mockActiveStatus,
  mockOverview,
  mockReloadTargets,
  mockBestSwitchExecute,
  mockSwitchExecute,
  mockSwitchPreview
} from "./lib/mock-data";

function isTauriAvailable(): boolean {
  return typeof window !== "undefined" && "__TAURI_INTERNALS__" in window;
}

function fallbackError<T>(message: string): DesktopCommandResult<T> {
  return {
    ok: false,
    error: {
      code: "desktop-shell-unavailable",
      message,
      retryable: true
    }
  };
}

function mockOk<T>(data: T): DesktopCommandResult<T> {
  return { ok: true, data };
}

export interface SmokeTraceSnapshot {
  phase: string;
  view: string;
  activeProfile: string | null;
  selectedLabel: string | null;
  profileCount: number;
  refreshCount: number;
  event: string;
}

export async function loadProfilesOverview(): Promise<
  DesktopCommandResult<ProfilesOverviewPayload>
> {
  if (!isTauriAvailable()) {
    return mockOk(mockOverview);
  }

  try {
    return await invoke<DesktopCommandResult<ProfilesOverviewPayload>>(
      "desktop_profiles_overview"
    );
  } catch {
    return fallbackError(
      "The native desktop bridge is unavailable, so shared switcher data could not be loaded."
    );
  }
}

export async function loadActiveProfileStatus(): Promise<
  DesktopCommandResult<ActiveProfileStatusPayload>
> {
  if (!isTauriAvailable()) {
    return mockOk(mockActiveStatus);
  }

  try {
    return await invoke<DesktopCommandResult<ActiveProfileStatusPayload>>(
      "desktop_active_profile_status"
    );
  } catch {
    return fallbackError(
      "The native desktop bridge is unavailable, so the active profile status could not be loaded."
    );
  }
}

export async function previewSwitch(
  profileLabel: string
): Promise<DesktopCommandResult<SwitchPreviewPayload>> {
  if (!profileLabel) {
    return fallbackError("Choose a profile before previewing a switch.");
  }

  if (!isTauriAvailable()) {
    return mockOk(mockSwitchPreview(profileLabel));
  }

  try {
    return await invoke<DesktopCommandResult<SwitchPreviewPayload>>(
      "desktop_switch_preview",
      {
        request: { profileLabel }
      }
    );
  } catch {
    return fallbackError(
      "The native desktop bridge is unavailable, so switch preview could not be loaded."
    );
  }
}

export async function executeSwitch(
  profileLabel: string
): Promise<DesktopCommandResult<SwitchExecutePayload>> {
  if (!profileLabel) {
    return fallbackError("Choose a profile before executing a switch.");
  }

  if (!isTauriAvailable()) {
    return mockOk(mockSwitchExecute(profileLabel));
  }

  try {
    return await invoke<DesktopCommandResult<SwitchExecutePayload>>(
      "desktop_switch_execute",
      {
        request: { profileLabel }
      }
    );
  } catch {
    return fallbackError(
      "The native desktop bridge is unavailable, so switch execution could not be completed."
    );
  }
}

export async function executeBestSwitch(): Promise<
  DesktopCommandResult<SwitchExecutePayload>
> {
  if (!isTauriAvailable()) {
    return mockOk(mockBestSwitchExecute());
  }

  try {
    return await invoke<DesktopCommandResult<SwitchExecutePayload>>(
      "desktop_switch_best_execute"
    );
  } catch {
    return fallbackError(
      "The native desktop bridge is unavailable, so direct best-profile switching could not be completed."
    );
  }
}

export async function loadReloadTargets(): Promise<
  DesktopCommandResult<ReloadTargetsPayload>
> {
  if (!isTauriAvailable()) {
    return mockOk(mockReloadTargets);
  }

  try {
    return await invoke<DesktopCommandResult<ReloadTargetsPayload>>(
      "desktop_reload_targets"
    );
  } catch {
    return fallbackError(
      "The native desktop bridge is unavailable, so reload targets could not be loaded."
    );
  }
}

export async function reloadTarget(
  target: ReloadTarget
): Promise<DesktopCommandResult<ReloadOutcomePayload>> {
  if (!isTauriAvailable()) {
    return mockOk({
      target,
      attempted: true,
      restarted: true,
      message: `Mock reload of ${target} completed successfully.`,
      manualHints: []
    });
  }

  try {
    return await invoke<DesktopCommandResult<ReloadOutcomePayload>>(
      "desktop_reload_target",
      {
        request: { target }
      }
    );
  } catch {
    return fallbackError(
      "The native desktop bridge is unavailable, so reload execution could not be requested."
    );
  }
}

export async function loadSmokeMode(): Promise<boolean> {
  if (!isTauriAvailable()) {
    return false;
  }

  try {
    return await invoke<boolean>("desktop_smoke_mode");
  } catch {
    return false;
  }
}

export async function loadDemoMode(): Promise<boolean> {
  if (!isTauriAvailable()) {
    return false;
  }

  try {
    return await invoke<boolean>("desktop_demo_mode");
  } catch {
    return false;
  }
}

export async function recordSmokeTrace(snapshot: SmokeTraceSnapshot): Promise<boolean> {
  if (!isTauriAvailable()) {
    return false;
  }

  try {
    const result = await invoke<DesktopCommandResult<boolean>>(
      "desktop_record_smoke_trace",
      { snapshot }
    );
    return result.ok ? result.data : false;
  } catch {
    return false;
  }
}

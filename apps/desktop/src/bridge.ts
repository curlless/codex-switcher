import { invoke } from "@tauri-apps/api/core";
import type {
  ActiveProfileStatusPayload,
  DesktopCommandResult,
  ProfilesOverviewPayload,
  ReloadOutcomePayload,
  ReloadTarget,
  ReloadTargetsPayload,
  SwitchPreviewPayload
} from "./lib/contracts";

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

export async function loadProfilesOverview(): Promise<
  DesktopCommandResult<ProfilesOverviewPayload>
> {
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

export async function loadReloadTargets(): Promise<
  DesktopCommandResult<ReloadTargetsPayload>
> {
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

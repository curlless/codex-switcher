import { invoke } from "@tauri-apps/api/core";
import type {
  ActionNotice,
  ActiveProfileStatusPayload,
  DesktopCommandResult,
  ProfilesOverviewPayload,
  ReloadTarget,
  ReloadTargetsPayload
} from "./lib/contracts";

const fallbackOverview: ProfilesOverviewPayload = {
  workspaceLabel: "Windows-first MVP shell",
  profiles: [
    {
      label: "work-pro",
      plan: "ChatGPT Pro",
      reserved: false,
      status: "active",
      sevenDayRemaining: "74%",
      fiveHourRemaining: "61%"
    },
    {
      label: "openclaw-raymond",
      plan: "Team",
      reserved: true,
      status: "reserved",
      sevenDayRemaining: "93%",
      fiveHourRemaining: "80%"
    },
    {
      label: "night-shift",
      plan: "Plus",
      reserved: false,
      status: "available",
      sevenDayRemaining: "41%",
      fiveHourRemaining: "34%"
    }
  ],
  events: [
    "Desktop shell scaffold ready",
    "Shared service extraction is the next story",
    "CLI remains the production path during MVP bootstrap"
  ],
  lastRefresh: "Mocked browser preview"
};

const fallbackStatus: ActiveProfileStatusPayload = {
  activeProfile: "work-pro",
  summary: "Pro profile is active and has enough headroom for another switching cycle.",
  reservedProfiles: 1
};

const fallbackReloadTargets: ReloadTargetsPayload = {
  targets: [
    {
      id: "codex",
      label: "Reload Codex",
      description: "Refresh the Codex desktop session after a profile change."
    },
    {
      id: "cursor",
      label: "Reload Cursor",
      description: "Refresh Cursor only when the switch affects editor-side auth state."
    }
  ],
  lastReloaded: "Not reloaded yet"
};

function fallbackError(message: string): DesktopCommandResult<never> {
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
    return { ok: true, data: fallbackOverview };
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
    return { ok: true, data: fallbackStatus };
  }
}

export async function previewSwitch(
  profileLabel: string
): Promise<DesktopCommandResult<ActionNotice>> {
  if (!profileLabel) {
    return fallbackError("Choose a profile before previewing a switch.");
  }

  try {
    return await invoke<DesktopCommandResult<ActionNotice>>(
      "desktop_switch_preview",
      {
        request: { profileLabel }
      }
    );
  } catch {
    return {
      ok: true,
      data: {
        title: "Preview switch",
        status: "placeholder",
        detail: `Browser preview is showing a mocked switch plan for ${profileLabel}.`
      }
    };
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
    return { ok: true, data: fallbackReloadTargets };
  }
}

export async function reloadTarget(
  target: ReloadTarget
): Promise<DesktopCommandResult<ActionNotice>> {
  try {
    return await invoke<DesktopCommandResult<ActionNotice>>(
      "desktop_reload_target",
      {
        request: { target }
      }
    );
  } catch {
    const label = target === "codex" ? "Codex" : "Cursor";
    return {
      ok: true,
      data: {
        title: `Reload ${label}`,
        status: "placeholder",
        detail: `${label} reload is stubbed in browser preview mode.`
      }
    };
  }
}

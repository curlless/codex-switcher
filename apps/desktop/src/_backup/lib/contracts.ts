export type ProfileStatus = "active" | "available" | "reserved";

export type ReloadTarget = "codex" | "cursor";

export interface DesktopCommandError {
  code: string;
  message: string;
  retryable: boolean;
}

export type DesktopCommandResult<T> =
  | {
      ok: true;
      data: T;
    }
  | {
      ok: false;
      error: DesktopCommandError;
    };

export interface ProfileCard {
  label: string;
  plan: string;
  reserved: boolean;
  status: ProfileStatus;
  sevenDayRemaining: string;
  fiveHourRemaining: string;
}

export interface ProfilesOverviewPayload {
  workspaceLabel: string;
  profiles: ProfileCard[];
  events: string[];
  lastRefresh: string;
}

export interface ActiveProfileStatusPayload {
  activeProfile: string;
  summary: string;
  reservedProfiles: number;
}

export interface SwitchPreviewRequest {
  profileLabel: string;
}

export interface SwitchProfilePayload {
  label: string;
  plan: string;
  reserved: boolean;
  status: ProfileStatus;
  current: boolean;
  recommended: boolean;
  rank: number | null;
  sevenDayRemaining: string;
  fiveHourRemaining: string;
  unavailableReason: string | null;
}

export interface SwitchPreviewPayload {
  requestedProfile: string;
  activeProfile: string | null;
  recommendedProfile: string | null;
  canSwitch: boolean;
  summary: string;
  profiles: SwitchProfilePayload[];
  manualHints: string[];
}

export interface ReloadTargetInfo {
  id: ReloadTarget;
  label: string;
  description: string;
}

export interface ReloadTargetsPayload {
  targets: ReloadTargetInfo[];
  lastReloaded: string;
}

export interface SwitchExecuteRequest {
  profileLabel: string;
}

export interface SwitchExecutePayload {
  switchedTo: string;
  previousProfile: string | null;
  success: boolean;
  summary: string;
  manualHints: string[];
}

export interface ReloadTargetRequest {
  target: ReloadTarget;
}

export interface ReloadOutcomePayload {
  target: string;
  attempted: boolean;
  restarted: boolean;
  message: string;
  manualHints: string[];
}

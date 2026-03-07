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

export interface ActionNotice {
  title: string;
  detail: string;
  status: "ok" | "placeholder";
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

export interface ReloadTargetRequest {
  target: ReloadTarget;
}

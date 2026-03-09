import type {
  ActiveProfileStatusPayload,
  ProfilesOverviewPayload,
  ReloadTargetsPayload,
  SwitchExecutePayload,
  SwitchPreviewPayload
} from "./contracts";

export const mockOverview: ProfilesOverviewPayload = {
  workspaceLabel: "Local workspace",
  profiles: [
    {
      label: "personal",
      plan: "Plus",
      reserved: false,
      status: "active",
      sevenDayRemaining: "72%",
      fiveHourRemaining: "45%"
    },
    {
      label: "work-main",
      plan: "Pro",
      reserved: false,
      status: "available",
      sevenDayRemaining: "91%",
      fiveHourRemaining: "88%"
    },
    {
      label: "work-backup",
      plan: "Pro",
      reserved: true,
      status: "reserved",
      sevenDayRemaining: "100%",
      fiveHourRemaining: "100%"
    },
    {
      label: "testing",
      plan: "Plus",
      reserved: false,
      status: "available",
      sevenDayRemaining: "34%",
      fiveHourRemaining: "12%"
    }
  ],
  events: [
    "Tauri shell scaffold serves real Rust switcher services",
    "React workspace reads profiles through typed bridge contracts",
    "Switch preview flow returns structured candidate data",
    "Reload actions execute through the shared Rust service layer"
  ],
  lastRefresh: new Date().toLocaleTimeString()
};

export const mockActiveStatus: ActiveProfileStatusPayload = {
  activeProfile: "personal",
  summary:
    "Active on personal (Plus). 7-day usage at 28%, 5-hour window at 55%. Two other profiles available for switching.",
  reservedProfiles: 1
};

export const mockReloadTargets: ReloadTargetsPayload = {
  targets: [
    {
      id: "codex",
      label: "Reload Codex",
      description: "Refresh the Codex desktop session after an account switch."
    },
    {
      id: "cursor",
      label: "Reload Cursor",
      description:
        "Refresh Cursor when the bootstrap shell updates editor-side auth."
    }
  ],
  lastReloaded: "Mock reload services ready."
};

export function mockSwitchPreview(
  profileLabel: string
): SwitchPreviewPayload {
  return {
    requestedProfile: profileLabel,
    activeProfile: "personal",
    recommendedProfile: "work-main",
    canSwitch: true,
    summary: `Preview switching from personal to ${profileLabel}`,
    profiles: [
      {
        label: "personal",
        plan: "Plus",
        reserved: false,
        status: "active",
        current: true,
        recommended: false,
        rank: 2,
        sevenDayRemaining: "72%",
        fiveHourRemaining: "45%",
        unavailableReason: null
      },
      {
        label: "work-main",
        plan: "Pro",
        reserved: false,
        status: "available",
        current: false,
        recommended: true,
        rank: 1,
        sevenDayRemaining: "91%",
        fiveHourRemaining: "88%",
        unavailableReason: null
      },
      {
        label: "work-backup",
        plan: "Pro",
        reserved: true,
        status: "reserved",
        current: false,
        recommended: false,
        rank: null,
        sevenDayRemaining: "100%",
        fiveHourRemaining: "100%",
        unavailableReason: "Profile is reserved and cannot be switched to directly."
      },
      {
        label: "testing",
        plan: "Plus",
        reserved: false,
        status: "available",
        current: false,
        recommended: false,
        rank: 3,
        sevenDayRemaining: "34%",
        fiveHourRemaining: "12%",
        unavailableReason: null
      }
    ],
    manualHints: []
  };
}

export function mockSwitchExecute(
  profileLabel: string
): SwitchExecutePayload {
  return {
    switchedTo: profileLabel,
    previousProfile: "personal",
    success: true,
    summary: `Switched active profile from personal to ${profileLabel}.`,
    manualHints: [
      "Consider reloading Codex to pick up the new credentials."
    ]
  };
}

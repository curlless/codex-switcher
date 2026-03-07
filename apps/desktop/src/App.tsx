import { useEffect, useState } from "react";
import {
  loadActiveProfileStatus,
  loadProfilesOverview,
  loadReloadTargets,
  previewSwitch,
  reloadTarget
} from "./bridge";
import type {
  ActionNotice,
  ActiveProfileStatusPayload,
  DesktopCommandError,
  ProfileCard,
  ProfilesOverviewPayload,
  ReloadTargetInfo,
  ReloadTargetsPayload
} from "./lib/contracts";

function ProfileList({
  profiles,
  selectedLabel,
  onSelect
}: {
  profiles: ProfileCard[];
  selectedLabel: string;
  onSelect: (label: string) => void;
}) {
  return (
    <aside className="sidebar">
      <div className="sidebar__top">
        <span className="eyebrow">Profiles</span>
        <button className="ghost-button">Import</button>
      </div>
      <div className="sidebar__list">
        {profiles.map((profile) => {
          const isSelected = profile.label === selectedLabel;
          return (
            <button
              key={profile.label}
              className={`profile-tile${isSelected ? " profile-tile--selected" : ""}`}
              onClick={() => onSelect(profile.label)}
              type="button"
            >
              <div className="profile-tile__row">
                <strong>{profile.label}</strong>
                <span className={`status-pill status-pill--${profile.status}`}>
                  {profile.status}
                </span>
              </div>
              <div className="profile-tile__meta">{profile.plan}</div>
              <div className="profile-tile__limits">
                <span>7d {profile.sevenDayRemaining}</span>
                <span>5h {profile.fiveHourRemaining}</span>
              </div>
            </button>
          );
        })}
      </div>
    </aside>
  );
}

export function App() {
  const [overview, setOverview] = useState<ProfilesOverviewPayload | null>(null);
  const [activeStatus, setActiveStatus] =
    useState<ActiveProfileStatusPayload | null>(null);
  const [reloadTargets, setReloadTargets] =
    useState<ReloadTargetsPayload | null>(null);
  const [selectedLabel, setSelectedLabel] = useState("");
  const [actionResult, setActionResult] = useState<ActionNotice | null>(null);
  const [commandError, setCommandError] = useState<DesktopCommandError | null>(
    null
  );

  useEffect(() => {
    let alive = true;

    async function bootstrapShell() {
      const [overviewResult, activeStatusResult, reloadTargetsResult] =
        await Promise.all([
          loadProfilesOverview(),
          loadActiveProfileStatus(),
          loadReloadTargets()
        ]);

      if (!alive) {
        return;
      }

      if (overviewResult.ok) {
        setOverview(overviewResult.data);
      } else {
        setCommandError(overviewResult.error);
      }

      if (activeStatusResult.ok) {
        setActiveStatus(activeStatusResult.data);
        setSelectedLabel(activeStatusResult.data.activeProfile);
      } else {
        setCommandError(activeStatusResult.error);
      }

      if (reloadTargetsResult.ok) {
        setReloadTargets(reloadTargetsResult.data);
      } else {
        setCommandError(reloadTargetsResult.error);
      }
    }

    void bootstrapShell();
    return () => {
      alive = false;
    };
  }, []);

  const selectedProfile =
    overview?.profiles.find((profile) => profile.label === selectedLabel) ?? null;

  async function handlePreviewSwitch() {
    const result = await previewSwitch(selectedLabel);
    if (result.ok) {
      setActionResult(result.data);
      setCommandError(null);
      return;
    }
    setCommandError(result.error);
  }

  async function handleReloadTarget(target: ReloadTargetInfo) {
    const result = await reloadTarget(target.id);
    if (result.ok) {
      setActionResult(result.data);
      setCommandError(null);
      return;
    }
    setCommandError(result.error);
  }

  return (
    <div className="app-shell">
      <header className="topbar">
        <div>
          <span className="eyebrow">Codex Switcher Desktop</span>
          <h1>Quiet profile control for heavy Codex usage</h1>
        </div>
        <div className="topbar__meta">
          <span className="workspace-chip">
            {overview?.workspaceLabel ?? "Bootstrapping workspace"}
          </span>
          <button className="primary-button" type="button">
            New profile
          </button>
        </div>
      </header>

      <main className="workspace">
        <ProfileList
          profiles={overview?.profiles ?? []}
          selectedLabel={selectedLabel}
          onSelect={setSelectedLabel}
        />

        <section className="content">
          <div className="hero-panel">
            <div>
              <span className="eyebrow">Active focus</span>
              <h2>{selectedProfile?.label ?? "Loading profile state"}</h2>
              <p>
                Desktop MVP keeps the CLI as source of truth and turns frequent
                save/load/switch/reload flows into a faster visual workspace.
              </p>
              <div className="hero-panel__summary">
                {activeStatus?.summary ?? "Reading the active profile contract."}
              </div>
            </div>
            <div className="hero-panel__metrics">
              <div>
                <span className="metric-label">7d headroom</span>
                <strong>{selectedProfile?.sevenDayRemaining ?? "--"}</strong>
              </div>
              <div>
                <span className="metric-label">5h headroom</span>
                <strong>{selectedProfile?.fiveHourRemaining ?? "--"}</strong>
              </div>
              <div>
                <span className="metric-label">Reserved profiles</span>
                <strong>{activeStatus?.reservedProfiles ?? 0}</strong>
              </div>
            </div>
          </div>

          <div className="grid">
            <article className="card">
              <span className="eyebrow">Actions</span>
              <h3>Switching flow</h3>
              <p>
                Start with the four actions that matter most. Everything else
                ships after the shared Rust service seam is stable.
              </p>
              <div className="button-row">
                <button
                  className="primary-button"
                  onClick={() => void handlePreviewSwitch()}
                  type="button"
                >
                  Preview switch
                </button>
              </div>
            </article>

            <article className="card">
              <span className="eyebrow">Reload targets</span>
              <h3>Native bridge placeholder</h3>
              <p>
                Codex app and Cursor reload actions stay in Rust and surface back
                into the UI as structured results.
              </p>
              <div className="button-row">
                {(reloadTargets?.targets ?? []).map((target) => (
                  <button
                    key={target.id}
                    className="ghost-button"
                    onClick={() => void handleReloadTarget(target)}
                    type="button"
                  >
                    {target.label}
                  </button>
                ))}
              </div>
            </article>
          </div>

          <div className="grid">
            <article className="card card--events">
              <span className="eyebrow">Roadmap signal</span>
              <h3>What this scaffold proves</h3>
              <ul className="event-list">
                {(overview?.events ?? []).map((event) => (
                  <li key={event}>{event}</li>
                ))}
              </ul>
            </article>

            <article className="card">
              <span className="eyebrow">Action result</span>
              <h3>{actionResult?.title ?? "No action yet"}</h3>
              <p>{actionResult?.detail ?? "Native commands are stubbed but wired."}</p>
              <div className="status-bar">
                <span>Status</span>
                <strong>{actionResult?.status ?? "idle"}</strong>
              </div>
            </article>
          </div>

          <section className="utility-strip" aria-label="Desktop status strip">
            <div className="utility-chip">
              <span className="metric-label">Last refresh</span>
              <strong>{overview?.lastRefresh ?? "Pending"}</strong>
            </div>
            <div className="utility-chip">
              <span className="metric-label">Reload lane</span>
              <strong>{reloadTargets?.lastReloaded ?? "Pending"}</strong>
            </div>
            <div className="utility-chip utility-chip--wide">
              <span className="metric-label">Bridge health</span>
              <strong>
                {commandError
                  ? `${commandError.code}: ${commandError.message}`
                  : "Typed placeholder commands are available for the shell bootstrap."}
              </strong>
            </div>
          </section>
        </section>
      </main>
    </div>
  );
}

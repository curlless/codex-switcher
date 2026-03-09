import { useEffect, useState } from "react";
import {
  executeSwitch,
  loadActiveProfileStatus,
  loadProfilesOverview,
  loadReloadTargets,
  previewSwitch,
  reloadTarget
} from "./bridge";
import { ProfileDetail } from "./components/ProfileDetail";
import { ProfileList } from "./components/ProfileList";
import { StatusStrip } from "./components/StatusStrip";
import { SwitchPanel } from "./components/SwitchPanel";
import { ToastContainer } from "./components/ToastContainer";
import type { Toast } from "./components/ToastContainer";
import type {
  ActiveProfileStatusPayload,
  DesktopCommandError,
  ProfilesOverviewPayload,
  ReloadTargetInfo,
  ReloadTargetsPayload,
  SwitchPreviewPayload,
  SwitchProfilePayload
} from "./lib/contracts";

type AppPhase = "loading" | "error" | "ready";

export function App() {
  const [overview, setOverview] = useState<ProfilesOverviewPayload | null>(null);
  const [activeStatus, setActiveStatus] = useState<ActiveProfileStatusPayload | null>(null);
  const [reloadTargets, setReloadTargets] = useState<ReloadTargetsPayload | null>(null);
  const [selectedLabel, setSelectedLabel] = useState("");
  const [commandError, setCommandError] = useState<DesktopCommandError | null>(null);
  const [phase, setPhase] = useState<AppPhase>("loading");

  const [switchPreview, setSwitchPreview] = useState<SwitchPreviewPayload | null>(null);
  const [switchLoading, setSwitchLoading] = useState(false);
  const [switchExecuting, setSwitchExecuting] = useState(false);

  const [toasts, setToasts] = useState<Toast[]>([]);
  const [toastCounter, setToastCounter] = useState(0);
  const [refreshing, setRefreshing] = useState(false);

  function addToast(status: Toast["status"], title: string, detail: string, hints: string[] = []) {
    const id = toastCounter + 1;
    setToastCounter(id);
    setToasts((prev) => [...prev, { id, status, title, detail, hints }]);
    setTimeout(() => { setToasts((prev) => prev.filter((t) => t.id !== id)); }, 5000);
  }

  function removeToast(id: number) {
    setToasts((prev) => prev.filter((t) => t.id !== id));
  }

  async function bootstrapShell(): Promise<boolean> {
    const [overviewResult, activeStatusResult, reloadTargetsResult] = await Promise.all([
      loadProfilesOverview(),
      loadActiveProfileStatus(),
      loadReloadTargets()
    ]);

    let hasData = false;
    let lastError: DesktopCommandError | null = null;

    if (overviewResult.ok) { setOverview(overviewResult.data); hasData = true; }
    else { lastError = overviewResult.error; }

    if (activeStatusResult.ok) { setActiveStatus(activeStatusResult.data); setSelectedLabel(activeStatusResult.data.activeProfile); hasData = true; }
    else { lastError = activeStatusResult.error; }

    if (reloadTargetsResult.ok) { setReloadTargets(reloadTargetsResult.data); }
    else { lastError = reloadTargetsResult.error; }

    setCommandError(lastError);
    setPhase(hasData ? "ready" : "error");
    return lastError === null;
  }

  useEffect(() => { bootstrapShell(); }, []);

  useEffect(() => {
    function onKey(e: KeyboardEvent) {
      if (e.key === "Escape") {
        if (switchPreview) setSwitchPreview(null);
        else if (toasts.length > 0) setToasts([]);
      }
    }
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, [switchPreview, toasts]);

  const selectedProfile = overview?.profiles.find((p) => p.label === selectedLabel) ?? null;
  const enrichedProfile: SwitchProfilePayload | null = switchPreview?.profiles.find((p) => p.label === selectedLabel) ?? null;

  async function handleRefresh() {
    setRefreshing(true);
    const ok = await bootstrapShell();
    setRefreshing(false);
    addToast(ok ? "success" : "warning", ok ? "Refreshed" : "Partial refresh", ok ? "Profile data updated." : "Some data could not be loaded.");
  }

  async function handlePreviewSwitch() {
    setSwitchLoading(true);
    const result = await previewSwitch(selectedLabel);
    setSwitchLoading(false);
    if (result.ok) { setSwitchPreview(result.data); setCommandError(null); }
    else { setCommandError(result.error); addToast("error", "Preview failed", result.error.message); }
  }

  async function handleExecuteSwitch(profileLabel: string) {
    setSwitchExecuting(true);
    const result = await executeSwitch(profileLabel);
    setSwitchExecuting(false);
    if (result.ok) {
      addToast(result.data.success ? "success" : "warning", result.data.success ? "Switched" : "Switch incomplete", result.data.summary, result.data.manualHints);
      setSwitchPreview(null);
      await bootstrapShell();
    } else {
      addToast("error", "Switch failed", result.error.message);
    }
  }

  async function handleReloadTarget(target: ReloadTargetInfo) {
    const result = await reloadTarget(target.id);
    if (result.ok) {
      addToast(result.data.restarted ? "success" : result.data.attempted ? "warning" : "error", target.label, result.data.message, result.data.manualHints);
      setCommandError(null);
    } else {
      addToast("error", "Reload failed", result.error.message);
    }
  }

  if (phase === "loading") {
    return (
      <div className="app-shell">
        <div className="loading-screen">
          <div className="loading-spinner" />
          <h2>Codex Switcher</h2>
          <p className="loading-hint">Connecting to bridge...</p>
        </div>
      </div>
    );
  }

  if (phase === "error" && !overview && !activeStatus) {
    return (
      <div className="app-shell">
        <div className="error-screen">
          <div className="error-screen__icon">!</div>
          <h2>Unable to connect</h2>
          <p className="error-screen__detail">
            {commandError ? `${commandError.code}: ${commandError.message}` : "Bridge is not responding."}
          </p>
          {commandError?.retryable !== false && (
            <button className="btn btn--primary" onClick={() => { setPhase("loading"); setCommandError(null); bootstrapShell(); }} type="button">
              Retry
            </button>
          )}
        </div>
      </div>
    );
  }

  return (
    <div className="app-shell">
      <header className="titlebar">
        <div className="titlebar__brand">
          <span className="titlebar__diamond">&#9670;</span>
          Codex Switcher
        </div>
        <div className="titlebar__right">
          <span className="titlebar__workspace">{overview?.workspaceLabel ?? "Workspace"}</span>
          <button
            className={`titlebar__btn${refreshing ? " titlebar__btn--spinning" : ""}`}
            onClick={() => void handleRefresh()}
            disabled={refreshing}
            type="button"
            aria-label="Refresh"
          >
            &#8635;
          </button>
        </div>
      </header>

      <main className="workspace">
        <ProfileList
          profiles={overview?.profiles ?? []}
          selectedLabel={selectedLabel}
          onSelect={setSelectedLabel}
        />

        <section className="main">
          <ProfileDetail
            profile={selectedProfile}
            enriched={enrichedProfile}
            summary={activeStatus?.summary ?? null}
            reservedProfiles={activeStatus?.reservedProfiles ?? 0}
          />

          <div className="divider" />

          <div className="actions-bar">
            <button
              className="btn btn--primary"
              onClick={() => void handlePreviewSwitch()}
              disabled={!selectedLabel || switchLoading}
              type="button"
            >
              {switchLoading ? "Loading..." : "Preview switch"}
            </button>
            {(reloadTargets?.targets ?? []).map((target) => (
              <button
                key={target.id}
                className="btn"
                onClick={() => void handleReloadTarget(target)}
                type="button"
                title={target.description}
              >
                {target.label}
              </button>
            ))}
          </div>

          {switchPreview && (
            <SwitchPanel
              preview={switchPreview}
              executing={switchExecuting}
              onExecute={(label) => void handleExecuteSwitch(label)}
              onDismiss={() => setSwitchPreview(null)}
            />
          )}
        </section>
      </main>

      <StatusStrip
        lastRefresh={overview?.lastRefresh ?? "—"}
        lastReloaded={reloadTargets?.lastReloaded ?? "—"}
        commandError={commandError}
      />

      <ToastContainer toasts={toasts} onDismiss={removeToast} />
    </div>
  );
}

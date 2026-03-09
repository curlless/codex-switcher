import { useEffect, useState } from "react";
import {
  executeSwitch,
  loadActiveProfileStatus,
  loadProfilesOverview,
  loadReloadTargets,
  previewSwitch,
  reloadTarget
} from "./bridge";
import { ActivityBar } from "./components/ActivityBar";
import type { ActivityView } from "./components/ActivityBar";
import { ProfileDetail } from "./components/ProfileDetail";
import { ProfileList } from "./components/ProfileList";
import { QuickSwitchView } from "./components/QuickSwitchView";
import { ReloadView } from "./components/ReloadView";
import { SettingsView } from "./components/SettingsView";
import type { AppSettings } from "./components/SettingsView";
import { StatusStrip } from "./components/StatusStrip";
import { SwitchPanel } from "./components/SwitchPanel";
import { ToastContainer } from "./components/ToastContainer";
import type { Toast } from "./components/ToastContainer";
import { t } from "./lib/i18n";
import type { Locale } from "./lib/i18n";
import { sortProfiles } from "./lib/sorting";
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

function loadSettings(): AppSettings {
  try {
    const saved = localStorage.getItem("codex-switcher-settings");
    if (saved) return { ...defaultSettings, ...JSON.parse(saved) };
  } catch {}
  return defaultSettings;
}

const defaultSettings: AppSettings = {
  locale: "en",
  sortMode: "rating",
  reloadAfterSwitch: false,
  primaryReloadTarget: "codex",
};

export function App() {
  const [overview, setOverview] = useState<ProfilesOverviewPayload | null>(null);
  const [activeStatus, setActiveStatus] = useState<ActiveProfileStatusPayload | null>(null);
  const [reloadTargets, setReloadTargets] = useState<ReloadTargetsPayload | null>(null);
  const [selectedLabel, setSelectedLabel] = useState("");
  const [commandError, setCommandError] = useState<DesktopCommandError | null>(null);
  const [phase, setPhase] = useState<AppPhase>("loading");
  const [activeView, setActiveView] = useState<ActivityView>("profiles");

  const [switchPreview, setSwitchPreview] = useState<SwitchPreviewPayload | null>(null);
  const [switchLoading, setSwitchLoading] = useState(false);
  const [switchExecuting, setSwitchExecuting] = useState(false);

  const [toasts, setToasts] = useState<Toast[]>([]);
  const [toastCounter, setToastCounter] = useState(0);
  const [refreshing, setRefreshing] = useState(false);
  const [recentActions, setRecentActions] = useState<string[]>([]);
  const [reloadingTargets, setReloadingTargets] = useState<Set<string>>(new Set());
  const [settings, setSettings] = useState<AppSettings>(loadSettings);

  const locale: Locale = settings.locale;

  function updateSettings(patch: Partial<AppSettings>) {
    setSettings((prev) => {
      const next = { ...prev, ...patch };
      localStorage.setItem("codex-switcher-settings", JSON.stringify(next));
      return next;
    });
  }

  function addRecent(action: string) {
    const ts = new Date().toLocaleTimeString([], { hour: "2-digit", minute: "2-digit" });
    setRecentActions((prev) => [`${ts} ${action}`, ...prev].slice(0, 5));
  }

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
      if ((e.ctrlKey || e.metaKey) && e.key === "r") { e.preventDefault(); handleRefresh(); }
      if ((e.ctrlKey || e.metaKey) && e.key === "1") { e.preventDefault(); setActiveView("profiles"); }
      if ((e.ctrlKey || e.metaKey) && e.key === "2") { e.preventDefault(); setActiveView("switch"); }
      if ((e.ctrlKey || e.metaKey) && e.key === "3") { e.preventDefault(); setActiveView("reload"); }
      if ((e.ctrlKey || e.metaKey) && e.key === "4") { e.preventDefault(); setActiveView("settings"); }
    }
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, [switchPreview, toasts]);

  const sortedProfiles = sortProfiles(overview?.profiles ?? [], settings.sortMode);
  const selectedProfile = sortedProfiles.find((p) => p.label === selectedLabel) ?? null;
  const enrichedProfile: SwitchProfilePayload | null = switchPreview?.profiles.find((p) => p.label === selectedLabel) ?? null;

  async function handleRefresh() {
    setRefreshing(true);
    const ok = await bootstrapShell();
    setRefreshing(false);
    addToast(ok ? "success" : "warning", t(locale, ok ? "refreshed" : "partialRefresh"), t(locale, ok ? "profileDataUpdated" : "someDataCouldNotBeLoaded"));
    addRecent(t(locale, "refreshedProfiles"));
  }

  async function handlePreviewSwitch() {
    setSwitchLoading(true);
    const result = await previewSwitch(selectedLabel);
    setSwitchLoading(false);
    if (result.ok) { setSwitchPreview(result.data); setCommandError(null); }
    else { setCommandError(result.error); addToast("error", t(locale, "previewFailed"), result.error.message); }
  }

  async function handleQuickSwitch(profileLabel: string) {
    setSwitchLoading(true);
    const result = await previewSwitch(profileLabel);
    setSwitchLoading(false);
    if (result.ok) {
      setSwitchPreview(result.data);
      setSelectedLabel(profileLabel);
      setCommandError(null);
    } else {
      setCommandError(result.error);
      addToast("error", t(locale, "previewFailed"), result.error.message);
    }
  }

  async function handleExecuteSwitch(profileLabel: string) {
    setSwitchExecuting(true);
    const result = await executeSwitch(profileLabel);
    setSwitchExecuting(false);
    if (result.ok) {
      addToast(result.data.success ? "success" : "warning", t(locale, result.data.success ? "switched" : "switchIncomplete"), result.data.summary, result.data.manualHints);
      setSwitchPreview(null);
      addRecent(`${t(locale, "switched")} → ${profileLabel}`);
      await bootstrapShell();
      if (settings.reloadAfterSwitch && reloadTargets?.targets) {
        const targets = settings.primaryReloadTarget === "all"
          ? reloadTargets.targets
          : reloadTargets.targets.filter((tt) => tt.id === settings.primaryReloadTarget);
        for (const target of targets) {
          void handleReloadTarget(target);
        }
      }
    } else {
      addToast("error", t(locale, "switchFailed"), result.error.message);
    }
  }

  async function handleReloadTarget(target: ReloadTargetInfo) {
    setReloadingTargets((prev) => new Set(prev).add(target.id));
    const result = await reloadTarget(target.id);
    setReloadingTargets((prev) => { const next = new Set(prev); next.delete(target.id); return next; });
    if (result.ok) {
      addToast(result.data.restarted ? "success" : result.data.attempted ? "warning" : "error", target.label, result.data.message, result.data.manualHints);
      setCommandError(null);
      addRecent(`${t(locale, "reload")} ${target.label}`);
    } else {
      addToast("error", t(locale, "reloadFailed"), result.error.message);
    }
  }

  function handleReserve(label: string, reserve: boolean) {
    if (!overview) return;
    const updated = {
      ...overview,
      profiles: overview.profiles.map((p) =>
        p.label === label
          ? { ...p, reserved: reserve, status: reserve ? "reserved" as const : "available" as const }
          : p
      ),
    };
    setOverview(updated);
    addToast("success", t(locale, reserve ? "reserveSuccess" : "unreserveSuccess"), t(locale, reserve ? "profileReserved" : "profileUnreserved"));
    addRecent(`${reserve ? t(locale, "reserve") : t(locale, "unreserve")} ${label}`);
  }

  if (phase === "loading") {
    return (
      <div className="app-shell">
        <div className="loading-screen">
          <div className="loading-spinner" />
          <h2>{t(locale, "appName")}</h2>
          <p className="loading-hint">{t(locale, "connectingToBridge")}</p>
        </div>
      </div>
    );
  }

  if (phase === "error" && !overview && !activeStatus) {
    return (
      <div className="app-shell">
        <div className="error-screen">
          <div className="error-screen__icon">!</div>
          <h2>{t(locale, "unableToConnect")}</h2>
          <p className="error-screen__detail">
            {commandError ? `${commandError.code}: ${commandError.message}` : t(locale, "bridgeNotResponding")}
          </p>
          {commandError?.retryable !== false && (
            <button className="btn btn--primary" onClick={() => { setPhase("loading"); setCommandError(null); bootstrapShell(); }} type="button">
              {t(locale, "retry")}
            </button>
          )}
        </div>
      </div>
    );
  }

  function renderMainContent() {
    if (activeView === "switch") {
      return (
        <>
          <QuickSwitchView
            profiles={sortedProfiles}
            activeProfile={activeStatus?.activeProfile ?? ""}
            onSwitch={handleQuickSwitch}
            switchLoading={switchLoading}
            locale={locale}
          />
          {switchPreview && (
            <>
              <div className="divider" />
              <SwitchPanel
                preview={switchPreview}
                executing={switchExecuting}
                onExecute={(label) => void handleExecuteSwitch(label)}
                onDismiss={() => setSwitchPreview(null)}
              />
            </>
          )}
        </>
      );
    }

    if (activeView === "reload") {
      return (
        <ReloadView
          targets={reloadTargets?.targets ?? []}
          lastReloaded={reloadTargets?.lastReloaded ?? "\u2014"}
          reloadingTargets={reloadingTargets}
          onReload={handleReloadTarget}
          locale={locale}
        />
      );
    }

    if (activeView === "settings") {
      return (
        <SettingsView
          settings={settings}
          onUpdate={updateSettings}
          locale={locale}
        />
      );
    }

    return (
      <>
        <ProfileDetail
          profile={selectedProfile}
          enriched={enrichedProfile}
          summary={activeStatus?.summary ?? null}
          reservedProfiles={activeStatus?.reservedProfiles ?? 0}
          workspaceLabel={overview?.workspaceLabel ?? t(locale, "workspace")}
          events={overview?.events ?? []}
          locale={locale}
          onReserve={handleReserve}
        />

        <div className="divider" />

        <div className="actions-bar">
          <button
            className="btn btn--primary"
            onClick={() => void handlePreviewSwitch()}
            disabled={!selectedLabel || switchLoading}
            type="button"
          >
            {switchLoading ? t(locale, "loading") : t(locale, "previewSwitch")}
          </button>
          {(reloadTargets?.targets ?? []).map((target) => (
            <button
              key={target.id}
              className="btn"
              onClick={() => void handleReloadTarget(target)}
              disabled={reloadingTargets.has(target.id)}
              type="button"
              title={target.description}
            >
              {reloadingTargets.has(target.id) ? `${target.label}...` : target.label}
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
      </>
    );
  }

  return (
    <div className="app-shell">
      <header className="titlebar">
        <div className="titlebar__brand">
          <span className="titlebar__diamond">{"\u25C6"}</span>
          {t(locale, "appName")}
        </div>
        <div className="titlebar__right">
          <span className="titlebar__workspace">{overview?.workspaceLabel ?? t(locale, "workspace")}</span>
          <button
            className={`titlebar__btn${refreshing ? " titlebar__btn--spinning" : ""}`}
            onClick={() => void handleRefresh()}
            disabled={refreshing}
            type="button"
            aria-label={t(locale, "refreshTooltip")}
            title={t(locale, "refreshTooltip")}
          >
            {"\u21BB"}
          </button>
        </div>
      </header>

      <main className="workspace">
        <ActivityBar activeView={activeView} onViewChange={setActiveView} locale={locale} />

        <ProfileList
          profiles={sortedProfiles}
          selectedLabel={selectedLabel}
          activeProfile={activeStatus?.activeProfile ?? ""}
          onSelect={(label) => { setSelectedLabel(label); if (activeView !== "profiles") setActiveView("profiles"); }}
          onReserve={handleReserve}
          recentActions={recentActions}
          locale={locale}
        />

        <section className="main">
          {renderMainContent()}
        </section>
      </main>

      <StatusStrip
        lastRefresh={overview?.lastRefresh ?? "\u2014"}
        lastReloaded={reloadTargets?.lastReloaded ?? "\u2014"}
        commandError={commandError}
        activeProfile={activeStatus?.activeProfile ?? ""}
        profileCount={sortedProfiles.length}
        view={activeView}
        locale={locale}
      />

      <ToastContainer toasts={toasts} onDismiss={removeToast} />
    </div>
  );
}

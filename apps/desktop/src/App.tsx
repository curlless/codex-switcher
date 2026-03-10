import { useEffect, useEffectEvent, useRef, useState } from "react";
import {
  executeBestSwitch,
  executeSwitch,
  previewSwitch,
  recordSmokeTrace,
  reloadTarget,
} from "./bridge";
import { ActivityBar } from "./components/ActivityBar";
import type { ActivityView } from "./components/ActivityBar";
import { ProfileDetail } from "./components/ProfileDetail";
import { ProfileList } from "./components/ProfileList";
import { QuickSwitchView } from "./components/QuickSwitchView";
import { ReloadView } from "./components/ReloadView";
import { SettingsView } from "./components/SettingsView";
import { StatusStrip } from "./components/StatusStrip";
import { SwitchPanel } from "./components/SwitchPanel";
import { ToastContainer } from "./components/ToastContainer";
import { useAppSettings } from "./hooks/useAppSettings";
import { useDesktopWindowState } from "./hooks/useDesktopWindowState";
import { useRecentActions } from "./hooks/useRecentActions";
import { useToasts } from "./hooks/useToasts";
import { useWorkspaceLayout } from "./hooks/useWorkspaceLayout";
import { t } from "./lib/i18n";
import type { Locale } from "./lib/i18n";
import { sortProfiles } from "./lib/sorting";
import {
  bootstrapWorkspaceShell,
  type AppPhase,
  type WorkspaceShellSnapshot,
  selectReloadTargets,
} from "./lib/workspace-shell";
import type {
  ActiveProfileStatusPayload,
  DesktopCommandError,
  ProfilesOverviewPayload,
  ReloadTargetInfo,
  ReloadTargetsPayload,
  SwitchPreviewPayload,
  SwitchProfilePayload,
} from "./lib/contracts";

function titleViewToken(activeView: ActivityView): string {
  switch (activeView) {
    case "profiles":
      return "profiles";
    case "switch":
      return "quick-switch";
    case "reload":
      return "reload";
    case "settings":
      return "settings";
  }
}

function buildWindowTitle({
  phase,
  activeView,
  overview,
  activeStatus,
  selectedLabel,
  refreshing,
  refreshCount,
  commandError,
}: {
  phase: AppPhase;
  activeView: ActivityView;
  overview: ProfilesOverviewPayload | null;
  activeStatus: ActiveProfileStatusPayload | null;
  selectedLabel: string;
  refreshing: boolean;
  refreshCount: number;
  commandError: DesktopCommandError | null;
}): string {
  const segments = ["Codex Switcher Desktop"];

  if (phase === "loading") {
    segments.push("phase:loading");
    return segments.join(" | ");
  }

  if (phase === "error" && !overview && !activeStatus) {
    segments.push("phase:error");
    if (commandError?.code) {
      segments.push(`error:${commandError.code}`);
    }
    return segments.join(" | ");
  }

  segments.push(`view:${titleViewToken(activeView)}`);
  if (activeStatus?.activeProfile) {
    segments.push(`active:${activeStatus.activeProfile}`);
  }
  if (selectedLabel) {
    segments.push(`selected:${selectedLabel}`);
  }
  if (overview) {
    segments.push(`profiles:${overview.profiles.length}`);
  }
  segments.push(`refresh:${refreshCount}`);
  if (refreshing) {
    segments.push("busy:refresh");
  }

  return segments.join(" | ");
}

export function App() {
  const [overview, setOverview] = useState<ProfilesOverviewPayload | null>(null);
  const [activeStatus, setActiveStatus] = useState<ActiveProfileStatusPayload | null>(null);
  const [reloadTargets, setReloadTargets] = useState<ReloadTargetsPayload | null>(null);
  const [selectedLabel, setSelectedLabel] = useState("");
  const [commandError, setCommandError] = useState<DesktopCommandError | null>(null);
  const [phase, setPhase] = useState<AppPhase>("loading");
  const [activeView, setActiveView] = useState<ActivityView>("switch");

  const [switchPreview, setSwitchPreview] = useState<SwitchPreviewPayload | null>(null);
  const [switchLoading, setSwitchLoading] = useState(false);
  const [switchExecuting, setSwitchExecuting] = useState(false);
  const [refreshing, setRefreshing] = useState(false);
  const [refreshCount, setRefreshCount] = useState(0);
  const [reloadingTargets, setReloadingTargets] = useState<Set<string>>(new Set());
  const [smokeMode, setSmokeMode] = useState(false);
  const [smokeSequenceStarted, setSmokeSequenceStarted] = useState(false);
  const [smokeEvent, setSmokeEvent] = useState("boot");

  const { settings, updateSettings } = useAppSettings();
  const { recentActions, addRecent } = useRecentActions();
  const { toasts, addToast, removeToast, clearToasts } = useToasts();
  const { profilePaneWidth, updateProfilePaneWidth } = useWorkspaceLayout();
  const resizeStateRef = useRef<{ startX: number; startWidth: number } | null>(null);
  const refreshPromiseRef = useRef<Promise<WorkspaceShellSnapshot> | null>(null);
  useDesktopWindowState();

  const locale: Locale = settings.locale;

  async function bootstrapShell() {
    const snapshot = await bootstrapWorkspaceShell({
      overview,
      activeStatus,
      reloadTargets,
      selectedLabel,
    });

    setOverview(snapshot.overview);
    setActiveStatus(snapshot.activeStatus);
    setReloadTargets(snapshot.reloadTargets);
    setSelectedLabel(snapshot.selectedLabel);
    setCommandError(snapshot.commandError);
    setPhase(snapshot.phase);
    setSmokeEvent(snapshot.ok ? "bootstrap-ready" : "bootstrap-partial");

    return snapshot;
  }

  const runShellRefresh = useEffectEvent(
    async (mode: "bootstrap" | "manual" | "poll" | "post-switch") => {
      if (refreshPromiseRef.current) {
        return await refreshPromiseRef.current;
      }

      const isManual = mode === "manual";
      if (isManual) {
        setRefreshing(true);
      }

      const refreshWork = (async () => {
        const snapshot = await bootstrapShell();
        setRefreshCount((prev) => prev + 1);
        setSmokeEvent(snapshot.ok ? "refresh-success" : "refresh-warning");

        if (mode === "manual") {
          addToast(
            snapshot.ok ? "success" : "warning",
            t(locale, snapshot.ok ? "refreshed" : "partialRefresh"),
            t(locale, snapshot.ok ? "profileDataUpdated" : "someDataCouldNotBeLoaded"),
          );
          addRecent(t(locale, "refreshedProfiles"));
        }

        return snapshot;
      })();

      refreshPromiseRef.current = refreshWork.finally(() => {
        refreshPromiseRef.current = null;
        if (isManual) {
          setRefreshing(false);
        }
      });

      return await refreshPromiseRef.current;
    },
  );

  useEffect(() => {
    void runShellRefresh("bootstrap");
  }, []);

  useEffect(() => {
    function onKey(event: KeyboardEvent) {
      if (event.key === "Escape") {
        if (switchPreview) {
          setSwitchPreview(null);
        } else if (toasts.length > 0) {
          clearToasts();
        }
      }

      if ((event.ctrlKey || event.metaKey) && event.key === "r") {
        event.preventDefault();
        void handleRefresh();
      }

      if ((event.ctrlKey || event.metaKey) && event.key === "1") {
        event.preventDefault();
        setActiveView("switch");
      }

      if ((event.ctrlKey || event.metaKey) && event.key === "2") {
        event.preventDefault();
        setActiveView("profiles");
      }

      if ((event.ctrlKey || event.metaKey) && event.key === "3") {
        event.preventDefault();
        setActiveView("reload");
      }

      if ((event.ctrlKey || event.metaKey) && event.key === "4") {
        event.preventDefault();
        setActiveView("settings");
      }
    }

    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, [clearToasts, switchPreview, toasts.length]);

  useEffect(() => {
    document.title = buildWindowTitle({
      phase,
      activeView,
      overview,
      activeStatus,
      selectedLabel,
      refreshing,
      refreshCount,
      commandError,
    });
  }, [
    activeStatus,
    activeView,
    commandError,
    overview,
    phase,
    refreshCount,
    refreshing,
    selectedLabel,
  ]);

  useEffect(() => {
    function handlePointerMove(event: MouseEvent) {
      if (!resizeStateRef.current) {
        return;
      }

      const nextWidth =
        resizeStateRef.current.startWidth + (event.clientX - resizeStateRef.current.startX);
      updateProfilePaneWidth(nextWidth);
    }

    function handlePointerUp() {
      if (!resizeStateRef.current) {
        return;
      }

      resizeStateRef.current = null;
      document.body.classList.remove("app--resizing");
    }

    window.addEventListener("mousemove", handlePointerMove);
    window.addEventListener("mouseup", handlePointerUp);

    return () => {
      window.removeEventListener("mousemove", handlePointerMove);
      window.removeEventListener("mouseup", handlePointerUp);
      resizeStateRef.current = null;
      document.body.classList.remove("app--resizing");
    };
  }, [updateProfilePaneWidth]);

  useEffect(() => {
    if (activeView === "profiles" || !resizeStateRef.current) {
      return;
    }

    resizeStateRef.current = null;
    document.body.classList.remove("app--resizing");
  }, [activeView]);

  useEffect(() => {
    if (phase === "loading") {
      return;
    }

    let ignore = false;

    async function syncSmokeTrace() {
      const wroteTrace = await recordSmokeTrace({
        phase,
        view: titleViewToken(activeView),
        activeProfile: activeStatus?.activeProfile ?? null,
        selectedLabel: selectedLabel || null,
        profileCount: overview?.profiles.length ?? 0,
        refreshCount,
        event: smokeEvent,
      });

      if (!ignore && wroteTrace && !smokeMode) {
        setSmokeMode(true);
      }
    }

    void syncSmokeTrace();

    return () => {
      ignore = true;
    };
  }, [
    activeStatus,
    activeView,
    overview,
    phase,
    refreshCount,
    selectedLabel,
    smokeEvent,
    smokeMode,
  ]);

  useEffect(() => {
    if (!smokeMode || smokeSequenceStarted || phase !== "ready") {
      return;
    }

    setSmokeSequenceStarted(true);

    async function runSmokeSequence() {
      await new Promise((resolve) => window.setTimeout(resolve, 1200));
      setSmokeEvent("switch-view");
      setActiveView("switch");
      await new Promise((resolve) => window.setTimeout(resolve, 1200));
      setSmokeEvent("reload-view");
      setActiveView("reload");
      await new Promise((resolve) => window.setTimeout(resolve, 1200));
      await handleRefresh();
    }

    void runSmokeSequence();
  }, [phase, smokeMode, smokeSequenceStarted]);

  const sortedProfiles = sortProfiles(overview?.profiles ?? [], settings.sortMode);
  const selectedProfile =
    sortedProfiles.find((profile) => profile.label === selectedLabel) ?? null;
  const enrichedProfile: SwitchProfilePayload | null =
    switchPreview?.profiles.find((profile) => profile.label === selectedLabel) ?? null;

  async function handleRefresh() {
    await runShellRefresh("manual");
  }

  async function handlePreviewSwitch() {
    setSwitchLoading(true);
    const result = await previewSwitch(selectedLabel);
    setSwitchLoading(false);

    if (result.ok) {
      setSwitchPreview(result.data);
      setCommandError(null);
      setSmokeEvent("preview-ready");
      return;
    }

    setCommandError(result.error);
    addToast("error", t(locale, "previewFailed"), result.error.message);
  }

  async function handleQuickSwitch() {
    setSwitchLoading(true);
    const result = await executeBestSwitch();
    setSwitchLoading(false);

    if (!result.ok) {
      setCommandError(result.error);
      addToast("error", t(locale, "switchFailed"), result.error.message);
      return;
    }

    setCommandError(null);
    setSwitchPreview(null);
    setSelectedLabel(result.data.switchedTo);
    addToast(
      result.data.success ? "success" : "warning",
      t(locale, result.data.success ? "switched" : "switchIncomplete"),
      result.data.summary,
      result.data.manualHints,
    );
    setSmokeEvent(result.data.success ? "quick-switch-executed" : "quick-switch-warning");
    addRecent(`${t(locale, "switched")} -> ${result.data.switchedTo}`);
    const refreshedShell = await runShellRefresh("post-switch");

    if (settings.reloadAfterSwitch) {
      const targets = selectReloadTargets(
        refreshedShell.reloadTargets,
        settings.primaryReloadTarget,
      );

      for (const target of targets) {
        void handleReloadTarget(target);
      }
    }
  }

  async function handleExecuteSwitch(profileLabel: string) {
    setSwitchExecuting(true);
    const result = await executeSwitch(profileLabel);
    setSwitchExecuting(false);

    if (!result.ok) {
      addToast("error", t(locale, "switchFailed"), result.error.message);
      return;
    }

    addToast(
      result.data.success ? "success" : "warning",
      t(locale, result.data.success ? "switched" : "switchIncomplete"),
      result.data.summary,
      result.data.manualHints,
    );
    setSwitchPreview(null);
    setSmokeEvent(result.data.success ? "switch-executed" : "switch-warning");
    addRecent(`${t(locale, "switched")} -> ${profileLabel}`);
    const refreshedShell = await runShellRefresh("post-switch");

    if (settings.reloadAfterSwitch) {
      const targets = selectReloadTargets(
        refreshedShell.reloadTargets,
        settings.primaryReloadTarget,
      );

      for (const target of targets) {
        void handleReloadTarget(target);
      }
    }
  }

  useEffect(() => {
    if (phase !== "ready") {
      return;
    }

    const timer = window.setInterval(() => {
      void runShellRefresh("poll");
    }, 60_000);

    return () => window.clearInterval(timer);
  }, [phase]);

  async function handleReloadTarget(target: ReloadTargetInfo) {
    setReloadingTargets((prev) => new Set(prev).add(target.id));
    const result = await reloadTarget(target.id);
    setReloadingTargets((prev) => {
      const next = new Set(prev);
      next.delete(target.id);
      return next;
    });

    if (result.ok) {
      addToast(
        result.data.restarted
          ? "success"
          : result.data.attempted
            ? "warning"
            : "error",
        target.label,
        result.data.message,
        result.data.manualHints,
      );
      setCommandError(null);
      setSmokeEvent(
        result.data.restarted
          ? `reload-${target.id}-success`
          : `reload-${target.id}-warning`,
      );
      addRecent(`${t(locale, "reload")} ${target.label}`);
      return;
    }

    addToast("error", t(locale, "reloadFailed"), result.error.message);
  }

  function handleReserve(label: string, reserve: boolean) {
    if (!overview) {
      return;
    }

    setOverview({
      ...overview,
      profiles: overview.profiles.map((profile) =>
        profile.label === label
          ? {
              ...profile,
              reserved: reserve,
              status: reserve ? "reserved" : "available",
            }
          : profile,
      ),
    });

    addToast(
      "warning",
      t(locale, reserve ? "reserveSuccess" : "unreserveSuccess"),
      t(locale, "localReserveNote"),
      [t(locale, reserve ? "profileReserved" : "profileUnreserved")],
    );
    addRecent(`${reserve ? t(locale, "reserve") : t(locale, "unreserve")} ${label}`);
  }

  function handleWorkspaceResizeStart(event: React.MouseEvent<HTMLDivElement>) {
    event.preventDefault();
    resizeStateRef.current = {
      startX: event.clientX,
      startWidth: profilePaneWidth,
    };
    document.body.classList.add("app--resizing");
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
            {commandError
              ? `${commandError.code}: ${commandError.message}`
              : t(locale, "bridgeNotResponding")}
          </p>
          {commandError?.retryable !== false && (
            <button
              className="btn btn--primary"
              onClick={() => {
                setPhase("loading");
                setCommandError(null);
                void bootstrapShell();
              }}
              type="button"
            >
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
        <QuickSwitchView
          profiles={overview?.profiles ?? []}
          activeProfile={activeStatus?.activeProfile ?? ""}
          onSwitch={handleQuickSwitch}
          switchLoading={switchLoading}
          locale={locale}
        />
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
            locale={locale}
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
          <span className="titlebar__workspace">
            {overview?.workspaceLabel ?? t(locale, "workspace")}
          </span>
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
        <ActivityBar
          activeView={activeView}
          onViewChange={setActiveView}
          locale={locale}
        />

        {activeView === "profiles" && (
          <>
            <ProfileList
              profiles={sortedProfiles}
              selectedLabel={selectedLabel}
              activeProfile={activeStatus?.activeProfile ?? ""}
              onSelect={(label) => {
                setSelectedLabel(label);
                if (activeView !== "profiles") {
                  setActiveView("profiles");
                }
              }}
              onReserve={handleReserve}
              recentActions={recentActions}
              locale={locale}
              paneWidth={profilePaneWidth}
            />
            <div
              className="workspace-resizer"
              onMouseDown={handleWorkspaceResizeStart}
              role="separator"
              aria-orientation="vertical"
              aria-label="Resize profile workspace sidebar"
            />
          </>
        )}

        <section className="main">{renderMainContent()}</section>
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

      <ToastContainer toasts={toasts} onDismiss={removeToast} locale={locale} />
    </div>
  );
}

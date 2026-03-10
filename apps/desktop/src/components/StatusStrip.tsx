import type { ActivityView } from "./ActivityBar";
import type { DesktopCommandError } from "../lib/contracts";
import type { Locale } from "../lib/i18n";
import { localizeRuntimeText, t } from "../lib/i18n";

const viewLabelKeys: Record<ActivityView, "profiles" | "quickSwitch" | "reload" | "settings"> = {
  profiles: "profiles",
  switch: "quickSwitch",
  reload: "reload",
  settings: "settings",
};

export function StatusStrip({
  lastRefresh,
  lastReloaded,
  commandError,
  activeProfile,
  profileCount,
  view,
  locale,
}: {
  lastRefresh: string;
  lastReloaded: string;
  commandError: DesktopCommandError | null;
  activeProfile: string;
  profileCount: number;
  view: ActivityView;
  locale: Locale;
}) {
  return (
    <footer className="statusbar" aria-label={t(locale, "statusBar")}>
      <div className="statusbar__item">
        <span className={`statusbar__dot statusbar__dot--${commandError ? "error" : "ok"}`} />
        <span>{commandError ? t(locale, "error") : t(locale, "connected")}</span>
      </div>
      {activeProfile && (
        <div className="statusbar__item">
          <span className="statusbar__active-profile">{activeProfile}</span>
        </div>
      )}
      <div className="statusbar__item">
        <span>{lastRefresh}</span>
      </div>
      <span className="statusbar__spacer" />
      <div className="statusbar__item">
        <span>{profileCount} {t(locale, "profiles").toLowerCase()}</span>
      </div>
      <div className="statusbar__item">
        <span className="statusbar__view-label">{t(locale, viewLabelKeys[view])}</span>
      </div>
      {lastReloaded && (
        <div className="statusbar__item" aria-live="polite">
          <span>{t(locale, "lastReloaded")}: {localizeRuntimeText(locale, lastReloaded)}</span>
        </div>
      )}
      {commandError && (
        <div className="statusbar__item" aria-live="polite">
          <span className="statusbar__error">
            {commandError.code}: {localizeRuntimeText(locale, commandError.message)}
          </span>
        </div>
      )}
    </footer>
  );
}

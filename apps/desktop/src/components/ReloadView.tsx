import type { ReloadTargetInfo } from "../lib/contracts";
import type { Locale } from "../lib/i18n";
import { t } from "../lib/i18n";

export function ReloadView({
  targets,
  lastReloaded,
  reloadingTargets,
  onReload,
  locale,
}: {
  targets: ReloadTargetInfo[];
  lastReloaded: string;
  reloadingTargets: Set<string>;
  onReload: (target: ReloadTargetInfo) => void;
  locale: Locale;
}) {
  return (
    <div className="reload-view">
      <div className="view-header">
        <h2 className="view-header__title">{t(locale, "reloadSessions")}</h2>
        <p className="view-header__desc">
          {t(locale, "reloadDesc")} {t(locale, "lastReloaded")}: {lastReloaded}
        </p>
      </div>

      <div className="reload-view__grid">
        {targets.map((target) => {
          const isLoading = reloadingTargets.has(target.id);

          return (
            <button
              key={target.id}
              className={`reload-view__card${isLoading ? " reload-view__card--loading" : ""}`}
              onClick={() => onReload(target)}
              disabled={isLoading}
              type="button"
              aria-label={`${t(locale, "reload")} ${target.label}`}
              aria-busy={isLoading}
            >
              <div className="reload-view__card-icon">
                {target.id === "codex" ? "\u25C8" : target.id === "cursor" ? "\u25C7" : "\u21BB"}
              </div>
              <div className="reload-view__card-info">
                <span className="reload-view__card-name">{target.label}</span>
                <span className="reload-view__card-desc">{target.description}</span>
              </div>
              <div className="reload-view__card-action">
                {isLoading ? (
                  <span className="reload-view__spinner" />
                ) : (
                  <span className="reload-view__card-arrow">{"\u21BB"}</span>
                )}
              </div>
            </button>
          );
        })}
      </div>

      {targets.length === 0 && (
        <div className="reload-view__empty">{t(locale, "noReloadTargets")}</div>
      )}

      <div className="reload-view__hint">{t(locale, "reloadTip")}</div>
    </div>
  );
}

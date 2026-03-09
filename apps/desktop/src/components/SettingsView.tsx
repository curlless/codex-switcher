import type { Locale } from "../lib/i18n";
import { t, getLocaleLabel } from "../lib/i18n";
import type { SortMode } from "../lib/sorting";

export interface AppSettings {
  locale: Locale;
  sortMode: SortMode;
  reloadAfterSwitch: boolean;
  primaryReloadTarget: "codex" | "cursor" | "all";
}

export function SettingsView({
  settings,
  onUpdate,
  locale,
}: {
  settings: AppSettings;
  onUpdate: (patch: Partial<AppSettings>) => void;
  locale: Locale;
}) {
  return (
    <div className="settings-view">
      <div className="view-header">
        <h2 className="view-header__title">{t(locale, "settingsTitle")}</h2>
        <p className="view-header__desc">{t(locale, "settingsDesc")}</p>
      </div>

      <div className="settings-view__sections">
        <div className="settings-section">
          <div className="settings-section__label">{t(locale, "language")}</div>
          <div className="settings-row">
            {(["en", "ru"] as Locale[]).map((loc) => (
              <button
                key={loc}
                className={`settings-chip${settings.locale === loc ? " settings-chip--active" : ""}`}
                onClick={() => onUpdate({ locale: loc })}
                type="button"
              >
                {getLocaleLabel(loc)}
              </button>
            ))}
          </div>
        </div>

        <div className="settings-section">
          <div className="settings-section__label">{t(locale, "sortBy")}</div>
          <div className="settings-row">
            {([
              { value: "rating" as SortMode, label: t(locale, "sortByRating") },
              { value: "name" as SortMode, label: t(locale, "sortByName") },
              { value: "usage" as SortMode, label: t(locale, "sortByUsage") },
            ]).map((opt) => (
              <button
                key={opt.value}
                className={`settings-chip${settings.sortMode === opt.value ? " settings-chip--active" : ""}`}
                onClick={() => onUpdate({ sortMode: opt.value })}
                type="button"
              >
                {opt.label}
              </button>
            ))}
          </div>
        </div>

        <div className="settings-section">
          <div className="settings-section__label">{t(locale, "reloadAfterSwitch")}</div>
          <p className="settings-section__desc">{t(locale, "reloadAfterSwitchDesc")}</p>
          <div className="settings-row">
            <button
              className={`settings-chip${settings.reloadAfterSwitch ? " settings-chip--active" : ""}`}
              onClick={() => onUpdate({ reloadAfterSwitch: true })}
              type="button"
            >
              {t(locale, "on")}
            </button>
            <button
              className={`settings-chip${!settings.reloadAfterSwitch ? " settings-chip--active" : ""}`}
              onClick={() => onUpdate({ reloadAfterSwitch: false })}
              type="button"
            >
              {t(locale, "off")}
            </button>
          </div>
        </div>

        <div className="settings-section">
          <div className="settings-section__label">{t(locale, "primaryReloadTarget")}</div>
          <p className="settings-section__desc">{t(locale, "primaryReloadTargetDesc")}</p>
          <div className="settings-row">
            {(["codex", "cursor", "all"] as const).map((target) => (
              <button
                key={target}
                className={`settings-chip${settings.primaryReloadTarget === target ? " settings-chip--active" : ""}`}
                onClick={() => onUpdate({ primaryReloadTarget: target })}
                type="button"
              >
                {target === "all" ? t(locale, "all") : target.charAt(0).toUpperCase() + target.slice(1)}
              </button>
            ))}
          </div>
        </div>
      </div>
    </div>
  );
}

import type { ProfileCard } from "../lib/contracts";
import type { Locale } from "../lib/i18n";
import { t } from "../lib/i18n";
import { profileScore } from "../lib/sorting";

function parsePercent(s: string): number {
  const n = parseInt(s, 10);
  return isNaN(n) ? 0 : n;
}

export function QuickSwitchView({
  profiles,
  activeProfile,
  onSwitch,
  switchLoading,
  locale,
}: {
  profiles: ProfileCard[];
  activeProfile: string;
  onSwitch: (label: string) => void;
  switchLoading: boolean;
  locale: Locale;
}) {
  const available = profiles.filter((p) => p.label !== activeProfile && p.status !== "reserved");
  const reserved = profiles.filter((p) => p.status === "reserved");
  const current = profiles.find((p) => p.label === activeProfile);

  return (
    <div className="quick-switch">
      <div className="view-header">
        <h2 className="view-header__title">{t(locale, "quickSwitchTitle")}</h2>
        <p className="view-header__desc">{t(locale, "quickSwitchDesc")}</p>
      </div>

      {current && (
        <div className="quick-switch__section">
          <div className="quick-switch__section-label">{t(locale, "current")}</div>
          <div className="quick-switch__card quick-switch__card--current">
            <span className="profile-item__dot profile-item__dot--active" />
            <div className="quick-switch__card-info">
              <span className="quick-switch__card-name">{current.label}</span>
              <span className="quick-switch__card-meta">{current.plan} &middot; 7d {current.sevenDayRemaining} &middot; 5h {current.fiveHourRemaining} &middot; {t(locale, "score")} {profileScore(current)}</span>
            </div>
            <span className="tag tag--status">active</span>
          </div>
        </div>
      )}

      {available.length > 0 && (
        <div className="quick-switch__section">
          <div className="quick-switch__section-label">{t(locale, "available")}</div>
          {available.map((p) => {
            const pct = parsePercent(p.sevenDayRemaining);
            const barColor = pct >= 50 ? "var(--green)" : pct >= 25 ? "var(--yellow)" : "var(--red)";
            const score = profileScore(p);
            return (
              <button
                key={p.label}
                className="quick-switch__card quick-switch__card--clickable"
                onClick={() => onSwitch(p.label)}
                disabled={switchLoading}
                type="button"
              >
                <span className={`profile-item__dot profile-item__dot--${p.status}`} />
                <div className="quick-switch__card-info">
                  <span className="quick-switch__card-name">{p.label}</span>
                  <span className="quick-switch__card-meta">{p.plan} &middot; 7d {p.sevenDayRemaining} &middot; 5h {p.fiveHourRemaining} &middot; {t(locale, "score")} {score}</span>
                  <span className="quick-switch__bar">
                    <span className="quick-switch__bar-fill" style={{ width: `${pct}%`, background: barColor }} />
                  </span>
                </div>
                <span className="quick-switch__arrow">{"\u2192"}</span>
              </button>
            );
          })}
        </div>
      )}

      {reserved.length > 0 && (
        <div className="quick-switch__section">
          <div className="quick-switch__section-label">{t(locale, "reserved")}</div>
          {reserved.map((p) => (
            <div key={p.label} className="quick-switch__card quick-switch__card--disabled">
              <span className="profile-item__dot profile-item__dot--reserved" />
              <div className="quick-switch__card-info">
                <span className="quick-switch__card-name">{p.label}</span>
                <span className="quick-switch__card-meta">{p.plan} &middot; {t(locale, "reservedForAnotherSession")}</span>
              </div>
            </div>
          ))}
        </div>
      )}

      {available.length === 0 && (
        <div className="quick-switch__empty">
          {t(locale, "noAvailableProfiles")}
        </div>
      )}

      <div className="quick-switch__hint">
        {t(locale, "quickSwitchTip")}
      </div>
    </div>
  );
}

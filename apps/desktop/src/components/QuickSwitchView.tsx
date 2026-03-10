import type { ProfileCard } from "../lib/contracts";
import type { Locale } from "../lib/i18n";
import { t } from "../lib/i18n";

function parsePercent(s: string): number {
  const n = parseInt(s, 10);
  return isNaN(n) ? 0 : n;
}

function firstBestCandidate(profiles: ProfileCard[], activeProfile: string): ProfileCard | null {
  return (
    profiles.find(
      (profile) =>
        profile.label !== activeProfile &&
        !profile.reserved &&
        profile.status === "available" &&
        !profile.availability,
    ) ?? null
  );
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
  onSwitch: () => void;
  switchLoading: boolean;
  locale: Locale;
}) {
  const reserved = profiles.filter((profile) => profile.status === "reserved");
  const current = profiles.find((profile) => profile.label === activeProfile);
  const bestCandidate = firstBestCandidate(profiles, activeProfile);
  const candidatePct = bestCandidate ? parsePercent(bestCandidate.sevenDayRemaining) : 0;
  const barColor =
    candidatePct >= 50 ? "var(--green)" : candidatePct >= 25 ? "var(--yellow)" : "var(--red)";

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
              <span className="quick-switch__card-meta">
                {current.plan} &middot; {t(locale, "sevenDayHeadroom")}{" "}
                {current.sevenDayRemaining} &middot; {t(locale, "fiveHourHeadroom")}{" "}
                {current.fiveHourRemaining}
              </span>
            </div>
            <span className="tag tag--status">{t(locale, "activeBadge")}</span>
          </div>
        </div>
      )}

      <div className="quick-switch__hero">
        <div className="quick-switch__section-label">{t(locale, "bestSwitchCandidate")}</div>
        {bestCandidate ? (
          <div className="quick-switch__card quick-switch__card--hero">
            <span className={`profile-item__dot profile-item__dot--${bestCandidate.status}`} />
            <div className="quick-switch__card-info">
              <span className="quick-switch__card-name">
                {bestCandidate.label}
                <span className="tag tag--recommended" style={{ marginLeft: 6 }}>
                  {t(locale, "recommended")}
                </span>
              </span>
              <span className="quick-switch__card-meta">
                {bestCandidate.plan} &middot; {t(locale, "sevenDayHeadroom")}{" "}
                {bestCandidate.sevenDayRemaining} &middot; {t(locale, "fiveHourHeadroom")}{" "}
                {bestCandidate.fiveHourRemaining}
              </span>
              <span className="quick-switch__bar">
                <span
                  className="quick-switch__bar-fill"
                  style={{ width: `${candidatePct}%`, background: barColor }}
                />
              </span>
            </div>
            <button
              className="btn btn--primary"
              onClick={() => onSwitch()}
              disabled={switchLoading}
              type="button"
            >
              {switchLoading ? t(locale, "switching") : t(locale, "switchBestProfile")}
            </button>
          </div>
        ) : (
          <div className="quick-switch__empty">{t(locale, "noAvailableProfiles")}</div>
        )}
      </div>

      {reserved.length > 0 && (
        <div className="quick-switch__section">
          <div className="quick-switch__section-label">{t(locale, "reserved")}</div>
          {reserved.map((profile) => (
            <div key={profile.label} className="quick-switch__card quick-switch__card--disabled">
              <span className="profile-item__dot profile-item__dot--reserved" />
              <div className="quick-switch__card-info">
                <span className="quick-switch__card-name">{profile.label}</span>
                <span className="quick-switch__card-meta">
                  {profile.plan} &middot; {t(locale, "reservedForAnotherSession")}
                </span>
              </div>
            </div>
          ))}
        </div>
      )}

      <div className="quick-switch__hint">{t(locale, "quickSwitchTip")}</div>
      <div className="quick-switch__hint">{t(locale, "localReserveNote")}</div>
    </div>
  );
}

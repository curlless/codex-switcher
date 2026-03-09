import type { ProfileCard } from "../lib/contracts";
import type { Locale } from "../lib/i18n";
import { t } from "../lib/i18n";

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
  const available = profiles.filter(
    (profile) => profile.label !== activeProfile && profile.status !== "reserved",
  );
  const reserved = profiles.filter((profile) => profile.status === "reserved");
  const current = profiles.find((profile) => profile.label === activeProfile);

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

      {available.length > 0 && (
        <div className="quick-switch__section">
          <div className="quick-switch__section-label">{t(locale, "available")}</div>
          {available.map((profile) => {
            const pct = parsePercent(profile.sevenDayRemaining);
            const barColor =
              pct >= 50 ? "var(--green)" : pct >= 25 ? "var(--yellow)" : "var(--red)";

            return (
              <button
                key={profile.label}
                className="quick-switch__card quick-switch__card--clickable"
                onClick={() => onSwitch(profile.label)}
                disabled={switchLoading}
                type="button"
              >
                <span className={`profile-item__dot profile-item__dot--${profile.status}`} />
                <div className="quick-switch__card-info">
                  <span className="quick-switch__card-name">
                    {profile.label}
                    {profile.availability && (
                      <span
                        className={`tag ${profile.availability.retryable ? "tag--availability-retryable" : "tag--availability-hard"}`}
                        style={{ marginLeft: 6 }}
                      >
                        {profile.availability.label}
                      </span>
                    )}
                  </span>
                  <span className="quick-switch__card-meta">
                    {profile.plan} &middot; {t(locale, "sevenDayHeadroom")}{" "}
                    {profile.sevenDayRemaining} &middot; {t(locale, "fiveHourHeadroom")}{" "}
                    {profile.fiveHourRemaining}
                  </span>
                  <span className="quick-switch__bar">
                    <span
                      className="quick-switch__bar-fill"
                      style={{ width: `${pct}%`, background: barColor }}
                    />
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

      {available.length === 0 && (
        <div className="quick-switch__empty">{t(locale, "noAvailableProfiles")}</div>
      )}

      <div className="quick-switch__hint">{t(locale, "quickSwitchTip")}</div>
      <div className="quick-switch__hint">{t(locale, "localReserveNote")}</div>
    </div>
  );
}

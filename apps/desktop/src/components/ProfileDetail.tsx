import type { ProfileCard, SwitchProfilePayload } from "../lib/contracts";
import type { Locale } from "../lib/i18n";
import { t } from "../lib/i18n";

function parsePercent(s: string): number {
  const n = parseInt(s, 10);
  return isNaN(n) ? 0 : n;
}

function meterVariant(pct: number): string {
  if (pct >= 50) return "ok";
  if (pct >= 25) return "warn";
  return "danger";
}

export function ProfileDetail({
  profile,
  enriched,
  summary,
  reservedProfiles,
  workspaceLabel,
  events,
  locale,
  onReserve,
}: {
  profile: ProfileCard | null;
  enriched: SwitchProfilePayload | null;
  summary: string | null;
  reservedProfiles: number;
  workspaceLabel: string;
  events: string[];
  locale: Locale;
  onReserve: (label: string, reserve: boolean) => void;
}) {
  if (!profile) {
    return (
      <div className="detail">
        <div className="breadcrumb">
          <span className="breadcrumb__item">{workspaceLabel}</span>
          <span className="breadcrumb__sep">/</span>
          <span className="breadcrumb__item breadcrumb__item--active">
            {t(locale, "selectProfile")}
          </span>
        </div>
        <div className="detail__placeholder">
          <p>{t(locale, "selectProfileHint")}</p>
        </div>
      </div>
    );
  }

  const recommended = enriched?.recommended ?? false;
  const availability = enriched?.availability ?? profile.availability ?? null;
  const pct7d = parsePercent(profile.sevenDayRemaining);
  const pct5h = parsePercent(profile.fiveHourRemaining);
  const statusTag =
    profile.status === "active"
      ? "tag--status"
      : profile.status === "reserved"
        ? "tag--status-reserved"
        : "tag--status-available";
  const eventsToRender =
    reservedProfiles > 0 && !summary
      ? [`${reservedProfiles} ${t(locale, "reserved").toLowerCase()}`, ...events].slice(0, 3)
      : events.slice(0, 3);

  return (
    <div className="detail">
      <div className="breadcrumb">
        <span className="breadcrumb__item">{workspaceLabel}</span>
        <span className="breadcrumb__sep">/</span>
        <span className="breadcrumb__item breadcrumb__item--active">{profile.label}</span>
      </div>

      <div className="detail__header">
        <span className="detail__name">{profile.label}</span>
        <div className="detail__tags">
          <span className={`tag ${statusTag}`}>
            {profile.status === "active"
              ? t(locale, "activeBadge")
              : profile.status === "reserved"
                ? t(locale, "reserved").toLowerCase()
                : t(locale, "available").toLowerCase()}
          </span>
          <span className="tag tag--plan">{profile.plan}</span>
          {recommended && <span className="tag tag--recommended">{t(locale, "recommended")}</span>}
          {availability && (
            <span
              className={`tag ${availability.retryable ? "tag--availability-retryable" : "tag--availability-hard"}`}
            >
              {availability.label}
            </span>
          )}
        </div>
      </div>

      <div className="meters">
        <div className="meter">
          <div className="meter__label">
            <span>{t(locale, "sevenDayHeadroom")}</span>
            <span className="meter__value">{profile.sevenDayRemaining}</span>
          </div>
          <div className="meter__track">
            <div
              className={`meter__fill meter__fill--${meterVariant(pct7d)}`}
              style={{ width: `${pct7d}%` }}
            />
          </div>
          {pct7d < 50 && (
            <div className="meter__hint">
              {pct7d >= 25 ? t(locale, "moderateUsage") : t(locale, "lowHeadroom")}
            </div>
          )}
        </div>
        <div className="meter">
          <div className="meter__label">
            <span>{t(locale, "fiveHourHeadroom")}</span>
            <span className="meter__value">{profile.fiveHourRemaining}</span>
          </div>
          <div className="meter__track">
            <div
              className={`meter__fill meter__fill--${meterVariant(pct5h)}`}
              style={{ width: `${pct5h}%` }}
            />
          </div>
          {pct5h < 50 && (
            <div className="meter__hint">
              {pct5h >= 25 ? t(locale, "windowNarrowing") : t(locale, "windowNearlyFull")}
            </div>
          )}
        </div>
      </div>

      {summary && <p className="detail__summary">{summary}</p>}

      {profile.status !== "active" && (
        <>
          <button
            className={`btn ${profile.reserved ? "btn--ghost" : "btn--outline-warn"}`}
            onClick={() => onReserve(profile.label, !profile.reserved)}
            type="button"
          >
            {t(locale, profile.reserved ? "clearLocalReserve" : "reserveLocally")}
          </button>
          <p className="detail__summary">{t(locale, "localReserveNote")}</p>
        </>
      )}

      {availability && (
        <div className="detail__warning">
          <span aria-hidden="true">!</span>
          {availability.reason}
        </div>
      )}

      {eventsToRender.length > 0 && (
        <div className="detail__events">
          <div className="detail__events-label">{t(locale, "recentEvents")}</div>
          {eventsToRender.map((event, index) => (
            <div key={index} className="detail__event">
              {event}
            </div>
          ))}
        </div>
      )}
    </div>
  );
}

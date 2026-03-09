import type { ProfileCard, SwitchProfilePayload } from "../lib/contracts";

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
  reservedProfiles
}: {
  profile: ProfileCard | null;
  enriched: SwitchProfilePayload | null;
  summary: string | null;
  reservedProfiles: number;
}) {
  if (!profile) {
    return (
      <div className="detail">
        <div className="detail__header">
          <span className="detail__name">Loading...</span>
        </div>
        <p className="detail__summary">Reading profile state from the bridge.</p>
      </div>
    );
  }

  const rank = enriched?.rank;
  const recommended = enriched?.recommended ?? false;
  const unavailableReason = enriched?.unavailableReason ?? null;
  const pct7d = parsePercent(profile.sevenDayRemaining);
  const pct5h = parsePercent(profile.fiveHourRemaining);

  const statusTag = profile.status === "active"
    ? "tag--status"
    : profile.status === "reserved"
      ? "tag--status-reserved"
      : "tag--status-available";

  return (
    <div className="detail">
      <div className="detail__header">
        <span className="detail__name">{profile.label}</span>
        <div className="detail__tags">
          <span className={`tag ${statusTag}`}>{profile.status}</span>
          <span className="tag tag--plan">{profile.plan}</span>
          {recommended && <span className="tag tag--recommended">recommended</span>}
          {rank !== null && rank !== undefined && <span className="tag tag--rank">#{rank}</span>}
          {reservedProfiles > 0 && (
            <span className="tag tag--status-reserved">{reservedProfiles} reserved</span>
          )}
        </div>
      </div>

      <div className="meters">
        <div className="meter">
          <div className="meter__label">
            <span>7-day headroom</span>
            <span className="meter__value">{profile.sevenDayRemaining}</span>
          </div>
          <div className="meter__track">
            <div className={`meter__fill meter__fill--${meterVariant(pct7d)}`} style={{ width: `${pct7d}%` }} />
          </div>
        </div>
        <div className="meter">
          <div className="meter__label">
            <span>5-hour headroom</span>
            <span className="meter__value">{profile.fiveHourRemaining}</span>
          </div>
          <div className="meter__track">
            <div className={`meter__fill meter__fill--${meterVariant(pct5h)}`} style={{ width: `${pct5h}%` }} />
          </div>
        </div>
      </div>

      {summary && <p className="detail__summary">{summary}</p>}

      {unavailableReason && (
        <div className="detail__warning">
          <span>!</span>
          {unavailableReason}
        </div>
      )}
    </div>
  );
}

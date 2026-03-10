import type { SwitchPreviewPayload } from "../lib/contracts";
import type { Locale } from "../lib/i18n";
import { getAvailabilityLabel, localizeRuntimeText, t } from "../lib/i18n";

export function SwitchPanel({
  preview,
  executing,
  onExecute,
  onDismiss,
  locale,
}: {
  preview: SwitchPreviewPayload;
  executing: boolean;
  onExecute: (profileLabel: string) => void;
  onDismiss: () => void;
  locale: Locale;
}) {
  return (
    <div className="switch-panel" role="region" aria-live="polite">
      <div className="switch-panel__header">
        <span className="switch-panel__title">{localizeRuntimeText(locale, preview.summary)}</span>
        <button
          className="switch-panel__close"
          onClick={onDismiss}
          type="button"
          aria-label={t(locale, "closePreview")}
        >
          &#10005;
        </button>
      </div>

      {preview.manualHints.length > 0 && (
        <ul className="switch-panel__hints">
          {preview.manualHints.map((hint) => (
            <li key={hint}>{localizeRuntimeText(locale, hint)}</li>
          ))}
        </ul>
      )}

      {preview.profiles.map((candidate) => {
        const isCurrent = candidate.current;
        const isUnavailable = !!candidate.availability;
        const dotColor =
          candidate.status === "active"
            ? "var(--green)"
            : candidate.status === "reserved"
              ? "var(--peach)"
              : "var(--text-dim)";

        return (
          <div
            key={candidate.label}
            className={`candidate-row${isCurrent ? " candidate-row--current" : ""}${isUnavailable ? " candidate-row--unavailable" : ""}`}
          >
            <span className="candidate-row__dot" style={{ background: dotColor }} />
            <div className="candidate-row__info">
              <div className="candidate-row__name">
                {candidate.label}
                {candidate.recommended && (
                  <span className="tag tag--recommended" style={{ marginLeft: 6 }}>
                    {t(locale, "recommended")}
                  </span>
                )}
                {candidate.availability && (
                  <span
                    className={`tag ${candidate.availability.retryable ? "tag--availability-retryable" : "tag--availability-hard"}`}
                    style={{ marginLeft: 6 }}
                  >
                    {getAvailabilityLabel(
                      locale,
                      candidate.availability.tag,
                      candidate.availability.label,
                    )}
                  </span>
                )}
                {isCurrent && (
                  <span className="tag tag--plan" style={{ marginLeft: 6 }}>
                    {t(locale, "current")}
                  </span>
                )}
              </div>
              <div className="candidate-row__meta">
                <span>{candidate.plan}</span>
                <span>{t(locale, "sevenDayHeadroom")} {candidate.sevenDayRemaining}</span>
                <span>{t(locale, "fiveHourHeadroom")} {candidate.fiveHourRemaining}</span>
                {candidate.rank !== null && (
                  <span>{t(locale, "rankLabel")} #{candidate.rank}</span>
                )}
              </div>
              {isUnavailable && (
                <div className="candidate-row__reason">
                  {candidate.availability
                    ? localizeRuntimeText(locale, candidate.availability.reason)
                    : ""}
                </div>
              )}
            </div>
            {!isCurrent && !isUnavailable && preview.canSwitch && (
              <button
                className="btn btn--primary btn--sm"
                onClick={() => onExecute(candidate.label)}
                disabled={executing}
                type="button"
              >
                {executing ? t(locale, "switching") : t(locale, "switchTo")}
              </button>
            )}
          </div>
        );
      })}
    </div>
  );
}

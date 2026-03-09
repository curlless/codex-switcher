import type { SwitchPreviewPayload } from "../lib/contracts";

export function SwitchPanel({
  preview,
  executing,
  onExecute,
  onDismiss
}: {
  preview: SwitchPreviewPayload;
  executing: boolean;
  onExecute: (profileLabel: string) => void;
  onDismiss: () => void;
}) {
  return (
    <div className="switch-panel">
      <div className="switch-panel__header">
        <span className="switch-panel__title">{preview.summary}</span>
        <button className="switch-panel__close" onClick={onDismiss} type="button" aria-label="Close preview">
          &#10005;
        </button>
      </div>

      {preview.manualHints.length > 0 && (
        <ul className="switch-panel__hints">
          {preview.manualHints.map((h) => <li key={h}>{h}</li>)}
        </ul>
      )}

      {preview.profiles.map((c) => {
        const isCurrent = c.current;
        const isUnavailable = !!c.unavailableReason;
        const dotColor = c.status === "active" ? "var(--green)"
          : c.status === "reserved" ? "var(--peach)" : "var(--text-dim)";

        return (
          <div key={c.label} className={`candidate-row${isCurrent ? " candidate-row--current" : ""}${isUnavailable ? " candidate-row--unavailable" : ""}`}>
            <span className="candidate-row__dot" style={{ background: dotColor }} />
            <div className="candidate-row__info">
              <div className="candidate-row__name">
                {c.label}
                {c.recommended && <span className="tag tag--recommended" style={{ marginLeft: 6 }}>recommended</span>}
                {isCurrent && <span className="tag tag--plan" style={{ marginLeft: 6 }}>current</span>}
              </div>
              <div className="candidate-row__meta">
                <span>{c.plan}</span>
                <span>7d {c.sevenDayRemaining}</span>
                <span>5h {c.fiveHourRemaining}</span>
                {c.rank !== null && <span>#{c.rank}</span>}
              </div>
              {isUnavailable && <div className="candidate-row__reason">{c.unavailableReason}</div>}
            </div>
            {!isCurrent && !isUnavailable && preview.canSwitch && (
              <button className="btn btn--primary btn--sm" onClick={() => onExecute(c.label)} disabled={executing} type="button">
                {executing ? "Switching..." : "Switch"}
              </button>
            )}
          </div>
        );
      })}
    </div>
  );
}

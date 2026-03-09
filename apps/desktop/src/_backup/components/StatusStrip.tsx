import type { DesktopCommandError } from "../lib/contracts";

export function StatusStrip({
  lastRefresh,
  lastReloaded,
  commandError
}: {
  lastRefresh: string;
  lastReloaded: string;
  commandError: DesktopCommandError | null;
}) {
  return (
    <footer className="statusbar" aria-label="Status bar">
      <div className="statusbar__item">
        <span className={`statusbar__dot statusbar__dot--${commandError ? "error" : "ok"}`} />
        <span>{commandError ? "Error" : "Connected"}</span>
      </div>
      <div className="statusbar__item">
        <span>Refreshed {lastRefresh}</span>
      </div>
      <div className="statusbar__item">
        <span>{lastReloaded}</span>
      </div>
      <span className="statusbar__spacer" />
      {commandError && (
        <div className="statusbar__item" aria-live="polite">
          <span>{commandError.code}: {commandError.message}</span>
        </div>
      )}
    </footer>
  );
}

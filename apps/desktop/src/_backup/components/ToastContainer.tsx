export interface Toast {
  id: number;
  status: "success" | "warning" | "error";
  title: string;
  detail: string;
  hints: string[];
}

export function ToastContainer({
  toasts,
  onDismiss
}: {
  toasts: Toast[];
  onDismiss: (id: number) => void;
}) {
  if (toasts.length === 0) return null;

  return (
    <div className="toast-container" aria-live="polite">
      {toasts.map((t) => (
        <div key={t.id} className={`toast toast--${t.status}`}>
          <div className="toast__header">
            <strong className="toast__title">{t.title}</strong>
            <button className="toast__close" onClick={() => onDismiss(t.id)} type="button" aria-label="Dismiss">&#10005;</button>
          </div>
          <p className="toast__detail">{t.detail}</p>
          {t.hints.length > 0 && (
            <ul className="toast__hints">
              {t.hints.map((h) => <li key={h}>{h}</li>)}
            </ul>
          )}
        </div>
      ))}
    </div>
  );
}

import type { Locale } from "../lib/i18n";
import { t } from "../lib/i18n";

export interface Toast {
  id: number;
  status: "success" | "warning" | "error";
  title: string;
  detail: string;
  hints: string[];
}

export function ToastContainer({
  toasts,
  onDismiss,
  locale,
}: {
  toasts: Toast[];
  onDismiss: (id: number) => void;
  locale: Locale;
}) {
  if (toasts.length === 0) return null;

  return (
    <div className="toast-container" aria-live="polite" aria-atomic="true">
      {toasts.map((toast) => (
        <div key={toast.id} className={`toast toast--${toast.status}`} role="status">
          <div className="toast__header">
            <strong className="toast__title">{toast.title}</strong>
            <button
              className="toast__close"
              onClick={() => onDismiss(toast.id)}
              type="button"
              aria-label={t(locale, "dismissToast")}
            >
              &#10005;
            </button>
          </div>
          <p className="toast__detail">{toast.detail}</p>
          {toast.hints.length > 0 && (
            <ul className="toast__hints">
              {toast.hints.map((hint) => (
                <li key={hint}>{hint}</li>
              ))}
            </ul>
          )}
        </div>
      ))}
    </div>
  );
}

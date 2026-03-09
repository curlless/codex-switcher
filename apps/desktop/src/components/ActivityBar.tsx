import type { Locale } from "../lib/i18n";
import { t } from "../lib/i18n";

export type ActivityView = "profiles" | "switch" | "reload" | "settings";

const icons: Record<ActivityView, { char: string; labelKey: "profiles" | "quickSwitch" | "reload" | "settings" }> = {
  profiles: { char: "\u2630", labelKey: "profiles" },
  switch: { char: "\u21C4", labelKey: "quickSwitch" },
  reload: { char: "\u21BB", labelKey: "reload" },
  settings: { char: "\u2699", labelKey: "settings" },
};

export function ActivityBar({
  activeView,
  onViewChange,
  locale,
}: {
  activeView: ActivityView;
  onViewChange: (view: ActivityView) => void;
  locale: Locale;
}) {
  const mainViews: ActivityView[] = ["profiles", "switch", "reload"];

  return (
    <nav className="activity-bar" aria-label="View navigation">
      <div className="activity-bar__top">
        {mainViews.map((view) => (
          <button
            key={view}
            className={`activity-bar__btn${view === activeView ? " activity-bar__btn--active" : ""}`}
            onClick={() => onViewChange(view)}
            type="button"
            aria-label={t(locale, icons[view].labelKey)}
            title={t(locale, icons[view].labelKey)}
          >
            {icons[view].char}
          </button>
        ))}
      </div>
      <div className="activity-bar__bottom">
        <button
          className={`activity-bar__btn${activeView === "settings" ? " activity-bar__btn--active" : ""}`}
          onClick={() => onViewChange("settings")}
          type="button"
          aria-label={t(locale, "settings")}
          title={t(locale, "settings")}
        >
          {icons.settings.char}
        </button>
      </div>
    </nav>
  );
}

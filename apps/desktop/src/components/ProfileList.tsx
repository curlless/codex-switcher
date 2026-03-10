import type { CSSProperties } from "react";
import { useEffect, useRef, useState } from "react";
import type { ProfileCard } from "../lib/contracts";
import type { Locale } from "../lib/i18n";
import { t } from "../lib/i18n";

function parsePercent(s: string): number {
  const n = parseInt(s, 10);
  return isNaN(n) ? 0 : n;
}

export function ProfileList({
  profiles,
  selectedLabel,
  activeProfile,
  onSelect,
  onReserve,
  recentActions,
  locale,
  paneWidth,
}: {
  profiles: ProfileCard[];
  selectedLabel: string;
  activeProfile: string;
  onSelect: (label: string) => void;
  onReserve: (label: string, reserve: boolean) => void;
  recentActions: string[];
  locale: Locale;
  paneWidth?: number;
}) {
  const listRef = useRef<HTMLDivElement>(null);
  const [focusedIndex, setFocusedIndex] = useState(-1);
  const [profilesOpen, setProfilesOpen] = useState(true);
  const [recentOpen, setRecentOpen] = useState(true);

  useEffect(() => {
    const idx = profiles.findIndex((profile) => profile.label === selectedLabel);
    setFocusedIndex(idx >= 0 ? idx : 0);
  }, [selectedLabel, profiles]);

  function handleKeyDown(event: React.KeyboardEvent) {
    if (profiles.length === 0) {
      return;
    }

    let next = focusedIndex;
    if (event.key === "ArrowDown") {
      event.preventDefault();
      next = Math.min(focusedIndex + 1, profiles.length - 1);
    } else if (event.key === "ArrowUp") {
      event.preventDefault();
      next = Math.max(focusedIndex - 1, 0);
    } else if (event.key === "Enter" || event.key === " ") {
      event.preventDefault();
      if (focusedIndex >= 0) {
        onSelect(profiles[focusedIndex].label);
      }
      return;
    } else {
      return;
    }

    setFocusedIndex(next);
    listRef.current?.querySelectorAll<HTMLButtonElement>("[role='option']")?.[next]?.focus();
  }

  return (
    <aside
      className="sidebar"
      style={
        paneWidth
          ? ({ "--profile-pane-width": `${paneWidth}px` } as CSSProperties)
          : undefined
      }
    >
      <button
        className="sidebar__section-toggle"
        onClick={() => setProfilesOpen(!profilesOpen)}
        type="button"
        aria-expanded={profilesOpen}
        aria-controls="sidebar-profiles"
      >
        <span className={`sidebar__chevron${profilesOpen ? " sidebar__chevron--open" : ""}`}>
          {"\u203A"}
        </span>
        <span className="sidebar__section-title">{t(locale, "profiles")}</span>
        <span className="sidebar__section-count">{profiles.length}</span>
      </button>

      <div id="sidebar-profiles">
        {profilesOpen &&
          (profiles.length === 0 ? (
            <div className="sidebar__empty">
              <p>{t(locale, "noProfilesFound")}</p>
              <span>{t(locale, "createProfileViaCli")}</span>
            </div>
          ) : (
            <div
              ref={listRef}
              className="sidebar__list"
              role="listbox"
              aria-label={t(locale, "profiles")}
              onKeyDown={handleKeyDown}
            >
              {profiles.map((profile, index) => {
                const isSelected = profile.label === selectedLabel;
                const isActive = profile.label === activeProfile;
                const pct = parsePercent(profile.sevenDayRemaining);
                const barColor =
                  pct >= 50 ? "var(--green)" : pct >= 25 ? "var(--yellow)" : "var(--red)";

                return (
                  <div key={profile.label} className="profile-item-wrapper">
                    <button
                      className={`profile-item${isSelected ? " profile-item--selected" : ""}`}
                      onClick={() => onSelect(profile.label)}
                      type="button"
                      role="option"
                      aria-selected={isSelected}
                      tabIndex={index === focusedIndex ? 0 : -1}
                    >
                      <span className={`profile-item__dot profile-item__dot--${profile.status}`} />
                      <span className="profile-item__info">
                        <span className="profile-item__name-row">
                          <span className="profile-item__name">{profile.label}</span>
                          {isActive && (
                            <span className="profile-item__active-badge">
                              {t(locale, "activeBadge")}
                            </span>
                          )}
                          {profile.reserved && (
                            <span className="profile-item__reserved-badge">
                              {t(locale, "reserved").toLowerCase()}
                            </span>
                          )}
                          {profile.availability && (
                            <span className="profile-item__availability-badge">
                              {profile.availability.label}
                            </span>
                          )}
                        </span>
                        <span className="profile-item__meta">
                          <span>{profile.plan}</span>
                          <span>{profile.sevenDayRemaining}</span>
                        </span>
                        <span className="profile-item__bar">
                          <span
                            className="profile-item__bar-fill"
                            style={{ width: `${pct}%`, background: barColor }}
                          />
                        </span>
                      </span>
                    </button>
                    {isSelected && !isActive && (
                      <button
                        className="profile-item__reserve-btn"
                        onClick={(event) => {
                          event.stopPropagation();
                          onReserve(profile.label, !profile.reserved);
                        }}
                        type="button"
                        aria-label={t(
                          locale,
                          profile.reserved ? "clearLocalReserve" : "reserveLocally",
                        )}
                        title={t(
                          locale,
                          profile.reserved ? "clearLocalReserve" : "reserveLocally",
                        )}
                      >
                        {t(locale, profile.reserved ? "clearLocalReserve" : "reserveLocally")}
                      </button>
                    )}
                  </div>
                );
              })}
            </div>
          ))}
      </div>

      <button
        className="sidebar__section-toggle"
        onClick={() => setRecentOpen(!recentOpen)}
        type="button"
        aria-expanded={recentOpen}
        aria-controls="sidebar-recent"
      >
        <span className={`sidebar__chevron${recentOpen ? " sidebar__chevron--open" : ""}`}>
          {"\u203A"}
        </span>
        <span className="sidebar__section-title">{t(locale, "recent")}</span>
        {recentActions.length > 0 && (
          <span className="sidebar__section-count">{recentActions.length}</span>
        )}
      </button>
      {recentOpen && (
        <div id="sidebar-recent" className="sidebar__recent">
          {recentActions.length === 0 ? (
            <div className="sidebar__recent-empty">{t(locale, "noRecentActions")}</div>
          ) : (
            recentActions.map((action, index) => (
              <div key={index} className="sidebar__recent-item">
                {action}
              </div>
            ))
          )}
        </div>
      )}
    </aside>
  );
}

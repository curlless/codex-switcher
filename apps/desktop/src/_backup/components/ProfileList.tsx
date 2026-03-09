import { useEffect, useRef, useState } from "react";
import type { ProfileCard } from "../lib/contracts";

export function ProfileList({
  profiles,
  selectedLabel,
  onSelect
}: {
  profiles: ProfileCard[];
  selectedLabel: string;
  onSelect: (label: string) => void;
}) {
  const listRef = useRef<HTMLDivElement>(null);
  const [focusedIndex, setFocusedIndex] = useState(-1);

  useEffect(() => {
    const idx = profiles.findIndex((p) => p.label === selectedLabel);
    setFocusedIndex(idx >= 0 ? idx : 0);
  }, [selectedLabel, profiles]);

  function handleKeyDown(e: React.KeyboardEvent) {
    if (profiles.length === 0) return;
    let next = focusedIndex;
    if (e.key === "ArrowDown") { e.preventDefault(); next = Math.min(focusedIndex + 1, profiles.length - 1); }
    else if (e.key === "ArrowUp") { e.preventDefault(); next = Math.max(focusedIndex - 1, 0); }
    else if (e.key === "Enter" || e.key === " ") { e.preventDefault(); if (focusedIndex >= 0) onSelect(profiles[focusedIndex].label); return; }
    else return;
    setFocusedIndex(next);
    listRef.current?.querySelectorAll<HTMLButtonElement>("[role='option']")?.[next]?.focus();
  }

  if (profiles.length === 0) {
    return (
      <aside className="sidebar">
        <div className="sidebar__header">Profiles</div>
        <div className="sidebar__empty">
          <p>No profiles found</p>
          <span>Create a profile via the CLI.</span>
        </div>
      </aside>
    );
  }

  return (
    <aside className="sidebar">
      <div className="sidebar__header">Profiles</div>
      <div ref={listRef} className="sidebar__list" role="listbox" aria-label="Profile list" onKeyDown={handleKeyDown}>
        {profiles.map((profile, index) => {
          const isSelected = profile.label === selectedLabel;
          return (
            <button
              key={profile.label}
              className={`profile-item${isSelected ? " profile-item--selected" : ""}`}
              onClick={() => onSelect(profile.label)}
              type="button"
              role="option"
              aria-selected={isSelected}
              tabIndex={index === focusedIndex ? 0 : -1}
            >
              <span className={`profile-item__dot profile-item__dot--${profile.status}`} />
              <span className="profile-item__info">
                <span className="profile-item__name">{profile.label}</span>
                <span className="profile-item__meta">
                  <span>{profile.plan}</span>
                  <span>{profile.sevenDayRemaining}</span>
                </span>
              </span>
            </button>
          );
        })}
      </div>
    </aside>
  );
}

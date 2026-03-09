import type { ProfileCard } from "./contracts";

export type SortMode = "rating" | "name" | "usage";

const SCORE_7D_WEIGHT = 70;
const SCORE_5H_WEIGHT = 30;

function parsePercent(s: string): number {
  const n = parseInt(s, 10);
  return isNaN(n) ? 0 : n;
}

export function profileScore(profile: ProfileCard): number {
  const pct7d = parsePercent(profile.sevenDayRemaining);
  const pct5h = parsePercent(profile.fiveHourRemaining);
  const tier = profileTier(profile);
  if (tier > 0) return 0;
  return (pct7d * SCORE_7D_WEIGHT) + (pct5h * SCORE_5H_WEIGHT);
}

function profileTier(profile: ProfileCard): number {
  const pct5h = parsePercent(profile.fiveHourRemaining);
  const pct7d = parsePercent(profile.sevenDayRemaining);
  if (pct7d > 0 && pct5h > 0) return 0;
  if (pct7d > 0 && pct5h === 0) return 1;
  return 2;
}

export function sortProfiles(profiles: ProfileCard[], mode: SortMode): ProfileCard[] {
  const sorted = [...profiles];

  if (mode === "name") {
    sorted.sort((a, b) => a.label.localeCompare(b.label));
    return sorted;
  }

  if (mode === "usage") {
    sorted.sort((a, b) => {
      const aUsage = parsePercent(a.sevenDayRemaining);
      const bUsage = parsePercent(b.sevenDayRemaining);
      return bUsage - aUsage;
    });
    return sorted;
  }

  sorted.sort((a, b) => {
    const aReserved = a.reserved ? 1 : 0;
    const bReserved = b.reserved ? 1 : 0;
    if (aReserved !== bReserved) return aReserved - bReserved;

    const aTier = profileTier(a);
    const bTier = profileTier(b);
    if (aTier !== bTier) return aTier - bTier;

    const aScore = profileScore(a);
    const bScore = profileScore(b);
    if (aScore !== bScore) return bScore - aScore;

    return a.label.localeCompare(b.label);
  });

  return sorted;
}

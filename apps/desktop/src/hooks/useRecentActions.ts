import { useState } from "react";

export function useRecentActions() {
  const [recentActions, setRecentActions] = useState<string[]>([]);

  function addRecent(action: string) {
    const ts = new Date().toLocaleTimeString([], {
      hour: "2-digit",
      minute: "2-digit",
    });

    setRecentActions((prev) => [`${ts} ${action}`, ...prev].slice(0, 5));
  }

  return {
    recentActions,
    addRecent,
  };
}

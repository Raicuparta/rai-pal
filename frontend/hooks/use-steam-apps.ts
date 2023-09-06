import { useCallback, useEffect, useState } from "react";
import { Game, getGameMap } from "@api/bindings";

export function useSteamApps() {
  const [steamApps, setSteamApps] = useState<Record<number, Game>>({});
  const [isLoading, setIsLoading] = useState(false);

  const updateSteamApps = useCallback((ignoreCache = false) => {
    setIsLoading(true);

    getGameMap()
      .then((gameMap) => {
        setSteamApps(gameMap);
      })
      .finally(() => setIsLoading(false));

    setIsLoading(false);
  }, []);

  const refresh = useCallback(() => {
    updateSteamApps(true);
  }, [updateSteamApps]);

  useEffect(() => {
    updateSteamApps();
  }, [updateSteamApps]);

  return [steamApps, isLoading, refresh] as const;
}

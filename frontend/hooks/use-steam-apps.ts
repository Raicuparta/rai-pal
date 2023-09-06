import { useCallback, useEffect, useState } from "react";
import { Game, getGameMap } from "@api/bindings";

export function useSteamApps() {
  const [steamApps, setSteamApps] = useState<Record<number, Game>>({});
  const [isLoading, setIsLoading] = useState(false);
  const [error, setError] = useState("");

  const updateSteamApps = useCallback((ignoreCache = false) => {
    setIsLoading(true);
    setError("");

    getGameMap()
      .then((gameMap) => {
        setSteamApps(gameMap);
      })
      .catch((error) => setError(`Failed to retrieve games: ${error}`))
      .finally(() => setIsLoading(false));

    setIsLoading(false);
  }, []);

  const refresh = useCallback(() => {
    updateSteamApps(true);
  }, [updateSteamApps]);

  useEffect(() => {
    updateSteamApps();
  }, [updateSteamApps]);

  return [steamApps, isLoading, refresh, error] as const;
}

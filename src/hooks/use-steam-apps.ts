import { useCallback, useEffect, useState } from "react";
import { GameMap } from "@api/game/game-map";
import { invoke } from "@tauri-apps/api";

export function useSteamApps() {
  const [steamApps, setSteamApps] = useState<GameMap>({});
  const [isLoading, setIsLoading] = useState(false);

  const updateSteamApps = useCallback((ignoreCache = false) => {
    setIsLoading(true);

    invoke("get_steam_apps_json")
      .then((json) => {
        console.log("json", json);
        setSteamApps(JSON.parse(json as string) as GameMap);
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

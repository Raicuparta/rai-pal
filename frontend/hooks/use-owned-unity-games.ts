import { useCallback, useEffect, useState } from "react";
import { OwnedUnityGame } from "@api/game/steam-owned-unity-games";
import { getOwnedGames } from "@api/bindings";

export const useOwnedUnityGames = () => {
  const [ownedUnityGames, setOwnedUnityGames] = useState<OwnedUnityGame[]>([]);
  const [isLoading, setIsLoading] = useState(false);

  const updateOwnedUnityGames = useCallback(async (ignoreCache = false) => {
    setIsLoading(true);
    getOwnedGames()
      .then(setOwnedUnityGames)
      .finally(() => {
        setIsLoading(false);
      });
    setOwnedUnityGames([]);
  }, []);

  const refresh = useCallback(() => {
    updateOwnedUnityGames(true);
  }, [updateOwnedUnityGames]);

  useEffect(() => {
    updateOwnedUnityGames();
  }, [updateOwnedUnityGames]);

  return [ownedUnityGames, isLoading, refresh] as const;
};

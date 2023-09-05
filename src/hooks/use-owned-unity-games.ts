import { useCallback, useEffect, useState } from "react";
import { OwnedUnityGame } from "@api/game/steam-owned-unity-games";

export const useOwnedUnityGames = () => {
  const [ownedUnityGames, setOwnedUnityGames] = useState<OwnedUnityGame[]>([]);
  const [isLoading, setIsLoading] = useState(false);

  const updateOwnedUnityGames = useCallback(async (ignoreCache = false) => {
    setIsLoading(true);
    setOwnedUnityGames([]);
    setIsLoading(false);
  }, []);

  const refresh = useCallback(() => {
    updateOwnedUnityGames(true);
  }, [updateOwnedUnityGames]);

  useEffect(() => {
    updateOwnedUnityGames();
  }, [updateOwnedUnityGames]);

  return [ownedUnityGames, isLoading, refresh] as const;
};

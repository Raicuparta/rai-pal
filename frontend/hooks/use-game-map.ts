import { Game, getGameMap, updateGameInfo } from "@api/bindings";
import { useCallback, useEffect, useState } from "react";

const defaultData = {};

export const useGameMap = () => {
  const [data, setData] = useState<Record<string, Game>>(defaultData);
  const [isLoading, setIsLoading] = useState(false);
  const [error, setError] = useState("");

  const updateData = useCallback((ignoreCache = false) => {
    setIsLoading(true);
    setError("");

    getGameMap(ignoreCache)
      .then(setData)
      .catch((error) => setError(`Failed to retrieve data: ${error}`))
      .finally(() => setIsLoading(false));
  }, []);

  const refresh = useCallback(() => {
    updateData(true);
  }, [updateData]);

  const refreshGame = useCallback((gameId: string) => {
    setIsLoading(true);
    setError("");

    updateGameInfo(gameId)
      .then(setData)
      .catch((error) => setError(`Failed to retrieve data: ${error}`))
      .finally(() => setIsLoading(false));
  }, []);

  useEffect(() => {
    updateData();
  }, [updateData]);

  return [data, isLoading, refresh, refreshGame, error] as const;
};

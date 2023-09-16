import { getGameMap, getModLoaders, getOwnedGames } from "@api/bindings";
import { useCallback, useEffect, useState } from "react";

type ApiFunction<TResultData> = (ignoreCache: boolean) => Promise<TResultData>;

function useBackendData<TData>(
  apiFunction: ApiFunction<TData>,
  defaultData: TData
) {
  const [data, setData] = useState<TData>(defaultData);
  const [isLoading, setIsLoading] = useState(false);
  const [error, setError] = useState("");

  const updateData = useCallback(
    (ignoreCache = false) => {
      setIsLoading(true);
      setError("");

      apiFunction(ignoreCache)
        .then(setData)
        .catch((error) => setError(`Failed to retrieve backend data: ${error}`))
        .finally(() => setIsLoading(false));
    },
    [apiFunction]
  );

  const refresh = useCallback(() => {
    updateData(true);
  }, [updateData]);

  useEffect(() => {
    updateData();
  }, [updateData]);

  return [data, isLoading, refresh, error] as const;
}

export const useGameMap = () => useBackendData(getGameMap, {});
export const useOwnedUnityGames = () => useBackendData(getOwnedGames, []);
export const useModLoaders = () => useBackendData(getModLoaders, []);

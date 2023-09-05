import { useCallback, useEffect, useState } from "react";
import { ModLoader } from "@api/mod-loader/mod-loader";

const modLoaders: ModLoader[] = [];

export function useModLoaders() {
  const [modLoaders, setModLoaders] = useState<ModLoader[]>([]);
  const [isLoading, setIsLoading] = useState(false);
  modLoaders.map;

  const updateModLoaders = useCallback(() => {
    setIsLoading(true);
    Promise.allSettled(
      modLoaders.map(async (modLoader) => {
        // TODO prevent initializing more than needed when called too frequently.
        // await modLoader.initialize();
      })
    )
      .then(() => setModLoaders(modLoaders))
      .then(() => setIsLoading(false));
  }, []);

  const refresh = useCallback(() => {
    for (const modLoader of modLoaders) {
      // modLoader.initialize(true);
    }
  }, []);

  useEffect(() => {
    updateModLoaders();
  }, [updateModLoaders]);

  return [modLoaders, isLoading, refresh] as const;
}

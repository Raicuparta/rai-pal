import { useCallback, useEffect, useState } from "react";
import { AppMap } from "@api/game/steam-game";

export function useSteamApps() {
  const [steamApps, setSteamApps] = useState<AppMap>({});
  const [isLoading, setIsLoading] = useState(false);

  const updateSteamApps = useCallback((ignoreCache = false) => {
    setIsLoading(true);

    setSteamApps({
      "0": {
        libraryPath: "/wrong",
        executables: [],
        id: "0",
        name: "GameName",
        distinctExecutables: [
          {
            architecture: "x64",
            fullPath: "/wrong",
            isLegacy: false,
            isLinux: true,
            modFilesPath: "/wrong",
            name: "ExeName",
            scriptingBackend: "il2cpp",
            unityVersion: "10",
            steamAppId: "0",
          },
        ],
      },
    });

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

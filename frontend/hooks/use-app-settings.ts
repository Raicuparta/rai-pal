import { AppSettings, commands } from "@api/bindings";
import { atom, useAtom } from "jotai";
import { useCallback, useEffect } from "react";

const defaultSettings: AppSettings = {
  hideGameThumbnails: false,
  overrideLanguage: null,
};

const appSettingsAtom = atom(defaultSettings);

export function useAppSettings() {
  const [settings, setSettingsInternal] = useAtom(appSettingsAtom);

  useEffect(() => {
    commands.getAppSettings().then((result) => {
      if (result.status === "ok") {
        setSettingsInternal(result.data);
      }
    });
  }, [setSettingsInternal]);

  const setSettings = useCallback(
    (newSettings: AppSettings) => {
      commands.saveAppSettings(newSettings).then((result) => {
        if (result.status === "ok") {
          setSettingsInternal(newSettings);
        }
      });
    },
    [setSettingsInternal],
  );

  return [settings, setSettings] as const;
}
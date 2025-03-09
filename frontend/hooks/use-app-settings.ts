import { AppSettings, commands } from "@api/bindings";
import { atom, useAtom } from "jotai";
import { useCallback, useEffect } from "react";

const defaultSettings: AppSettings = {
	hideGameThumbnails: false,
	overrideLanguage: null,
};

const appSettingsAtom = atom({
	isInitialized: false,
	settings: defaultSettings,
});

export function useAppSettings() {
	const [settings, setSettingsInternal] = useAtom(appSettingsAtom);

	useEffect(() => {
		if (settings.isInitialized) return;

		commands.getAppSettings().then((result) => {
			if (result.status === "ok") {
				setSettingsInternal({ isInitialized: true, settings: result.data });
			}
		});
	}, [setSettingsInternal, settings]);

	const setSettings = useCallback(
		(newSettings: AppSettings) => {
			commands.saveAppSettings(newSettings).then((result) => {
				if (result.status === "ok") {
					setSettingsInternal({ isInitialized: true, settings: newSettings });
				}
			});
		},
		[setSettingsInternal],
	);

	return [settings.settings, setSettings] as const;
}

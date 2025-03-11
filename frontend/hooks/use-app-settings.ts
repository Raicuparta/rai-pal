import { AppSettings, commands } from "@api/bindings";
import { atom, useAtom } from "jotai";
import { useEffect } from "react";

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

		commands.getAppSettings().then((initialSettings) => {
			setSettingsInternal({ isInitialized: true, settings: initialSettings });
		});
	}, [setSettingsInternal, settings]);

	const setSettings = (newSettings: AppSettings) => {
		commands.saveAppSettings(newSettings).then(() => {
			setSettingsInternal({ isInitialized: true, settings: newSettings });
		});
	};

	return [settings.settings, setSettings] as const;
}

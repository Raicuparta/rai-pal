import { AppSettings, commands } from "@api/bindings";
import { atom, useAtom } from "jotai";
import { useCallback, useEffect, useRef } from "react";

const defaultSettings: AppSettings = {
	hideGameThumbnails: false,
	overrideLanguage: null,
	gamesQuery: null,
	selectedTab: "Games",
	skipConfirmDialogs: [],
};

const appSettingsAtom = atom({
	isInitialized: false,
	settings: defaultSettings,
});

export function useAppSettings() {
	const [settings, setSettingsInternal] = useAtom(appSettingsAtom);
	const settingsRef = useRef(settings);

	useEffect(() => {
		if (settings.isInitialized) return;

		commands.getAppSettings().then((initialSettings) => {
			setSettingsInternal({ isInitialized: true, settings: initialSettings });
		});
	}, [setSettingsInternal, settings]);

	useEffect(() => {
		settingsRef.current = settings;
	}, [settings]);

	const setSettings = useCallback(
		(
			newSettingsGetter:
				| AppSettings
				| ((prevSettings: AppSettings) => AppSettings),
		) => {
			const newSettings =
				typeof newSettingsGetter === "function"
					? newSettingsGetter(settingsRef.current.settings)
					: newSettingsGetter;

			commands.saveAppSettings(newSettings).then(() => {
				setSettingsInternal({ isInitialized: true, settings: newSettings });
			});
		},
		[setSettingsInternal],
	);

	const reset = useCallback(() => {
		setSettings(defaultSettings);
	}, [setSettings]);

	return [settings.settings, setSettings, reset] as const;
}

import { AppSettings, commands } from "@api/bindings";
import { atom, useAtom, useStore } from "jotai";
import { useCallback, useEffect } from "react";
import { defaultSettings } from "./default-settings";

const appSettingsAtom = atom({
	isInitialized: false,
	settings: defaultSettings,
});

let settingsPromise: Promise<AppSettings> | null = null;

export function useAppSettings() {
	const [state, setSettingsInternal] = useAtom(appSettingsAtom);
	const store = useStore();

	useEffect(() => {
		if (state.isInitialized) return;

		if (!settingsPromise) {
			settingsPromise = commands.getAppSettings().catch((error) => {
				settingsPromise = null;
				throw error;
			});
		}

		let isActive = true;
		settingsPromise
			.then((initialSettings) => {
				if (!isActive) return;
				setSettingsInternal({
					isInitialized: true,
					settings: initialSettings,
				});
			})
			.catch((error) => {
				console.error(`Failed to load app settings: ${error}`);
			});

		return () => {
			isActive = false;
		};
	}, [state.isInitialized, setSettingsInternal]);

	const setSettings = useCallback(
		async (
			newSettingsGetter:
				AppSettings | ((prevSettings: AppSettings) => AppSettings),
		) => {
			const currentState = store.get(appSettingsAtom).settings;

			const newSettings =
				typeof newSettingsGetter === "function"
					? newSettingsGetter(currentState)
					: newSettingsGetter;

			setSettingsInternal({ isInitialized: true, settings: newSettings });

			await commands.saveAppSettings(newSettings);
		},
		[store, setSettingsInternal],
	);

	const reset = useCallback(() => {
		setSettings(defaultSettings);
	}, [setSettings]);

	return [state.settings, setSettings, reset] as const;
}

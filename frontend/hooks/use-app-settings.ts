import { AppSettings, commands } from "@api/bindings";
import { atom, useAtom, useStore } from "jotai";
import { useCallback, useEffect } from "react";

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

let isFetching = false;

export function useAppSettings() {
	const [state, setSettingsInternal] = useAtom(appSettingsAtom);
	const store = useStore();

	useEffect(() => {
		if (state.isInitialized || isFetching) return;

		isFetching = true;
		commands.getAppSettings().then((initialSettings) => {
			setSettingsInternal({ isInitialized: true, settings: initialSettings });
			isFetching = false;
		});
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

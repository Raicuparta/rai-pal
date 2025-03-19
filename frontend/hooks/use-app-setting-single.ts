import { AppSettings } from "@api/bindings";
import { useAppSettings } from "./use-app-settings";

export function useAppSettingSingle<TKey extends keyof AppSettings>(key: TKey) {
	const [settings, setSettings] = useAppSettings();

	const setValue = (
		newValueGetter:
			| AppSettings[TKey]
			| ((newValue: AppSettings[TKey]) => AppSettings[TKey]),
	) => {
		const newValue =
			typeof newValueGetter === "function"
				? newValueGetter(settings[key])
				: newValueGetter;

		setSettings((prevSettings) => ({
			...prevSettings,
			[key]: newValue,
		}));
	};

	return [settings[key], setValue] as const;
}

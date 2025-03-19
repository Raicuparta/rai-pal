import { AppSettings } from "@api/bindings";
import { useAppSettings } from "./use-app-settings";

export function useAppSettingSingle<TKey extends keyof AppSettings>(
	key: TKey,
): [AppSettings[TKey], (value: AppSettings[TKey]) => void] {
	const [settings, setSettings] = useAppSettings();

	const setValue = (newValue: AppSettings[TKey]) => {
		setSettings((prevSettings) => ({
			...prevSettings,
			[key]: newValue,
		}));
	};

	return [settings[key], setValue];
}

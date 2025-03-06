import { AppLocale } from "@api/bindings";
import { useAppSettings } from "@hooks/use-app-settings";
import { Select, Stack, Switch } from "@mantine/core";
import { translations } from "../../translations/translations";
import { useAtomValue } from "jotai";
import { detectedLocaleAtom } from "@hooks/use-translations";

const locales: AppLocale[] = [
	"DeDe",
	"EnUs",
	"EsEs",
	"FrFr",
	"JaJp",
	"KoKr",
	"PtPt",
	"ZhCn",
];

export function AppSettings() {
	const [settings, setSettings] = useAppSettings();
	const detectedLocale = useAtomValue(detectedLocaleAtom);

	const localeSelectValues = locales.sort().map((locale) => ({
		value: locale as string,
		label: translations[locale].meta.nativeName,
	}));

	if (detectedLocale) {
		localeSelectValues.unshift({
			value: "",
			label: `Auto-detected (${translations[detectedLocale].meta.nativeName})`,
		});
	}

	return (
		<Stack>
			<Switch
				label="Show game thumbnails on list"
				checked={!settings.hideGameThumbnails}
				onChange={(event) => {
					setSettings({
						...settings,
						hideGameThumbnails: !event.currentTarget.checked,
					});
				}}
			/>
			<Select
				label="Language"
				value={settings.overrideLanguage ?? ""}
				data={localeSelectValues}
				onChange={(value) => {
					setSettings({
						...settings,
						overrideLanguage: (value || null) as AppLocale | null,
					});
				}}
			/>
		</Stack>
	);
}

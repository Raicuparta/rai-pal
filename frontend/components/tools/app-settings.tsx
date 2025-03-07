import { AppLocale } from "@api/bindings";
import { useAppSettings } from "@hooks/use-app-settings";
import { NativeSelect, Stack, Switch, Group, Box } from "@mantine/core";
import { translations } from "../../translations/translations";
import { useAtomValue } from "jotai";
import { detectedLocaleAtom, useGetTranslated } from "@hooks/use-translations";
import { IconLanguage } from "@tabler/icons-react";

const locales: AppLocale[] = [
	"EnUs",
	"DeDe",
	"EsEs",
	"FrFr",
	"JaJp",
	"KoKr",
	"PtPt",
	"ZhCn",
];

export function AppSettings() {
	const t = useGetTranslated("appSettings");
	const [settings, setSettings] = useAppSettings();
	const detectedLocale = useAtomValue(detectedLocaleAtom);

	const localeSelectValues = locales.map((locale) => ({
		value: locale as string,
		label: translations[locale].meta.nativeName,
	}));

	return (
		<Stack>
			<Switch
				label={t("showGameThumbnails")}
				checked={!settings.hideGameThumbnails}
				onChange={(event) => {
					setSettings({
						...settings,
						hideGameThumbnails: !event.currentTarget.checked,
					});
				}}
			/>
			<NativeSelect
				label={
					<Group>
						<span>{t("language")}</span>
						<IconLanguage />
						<Box opacity={0.5}>Language</Box>
					</Group>
				}
				value={settings.overrideLanguage ?? ""}
				data={localeSelectValues}
				onChange={(event) => {
					setSettings({
						...settings,
						overrideLanguage: (event.currentTarget.value ||
							null) as AppLocale | null,
					});
				}}
			>
				{detectedLocale && (
					<option value="">
						{t("autoDetectedLanguage", {
							languageName: translations[detectedLocale].meta.nativeName,
						})}
					</option>
				)}
				{locales.map((locale) => (
					<option
						key={locale}
						value={locale}
					>
						{translations[locale].meta.nativeName}
					</option>
				))}
			</NativeSelect>
		</Stack>
	);
}

import { AppLocale, commands } from "@api/bindings";
import { useAppSettings } from "@hooks/use-app-settings";
import {
	NativeSelect,
	Group,
	Box,
	Menu,
	Stack,
	Tooltip,
	Divider,
	Button,
} from "@mantine/core";
import { useAtomValue } from "jotai";
import { detectedLocaleAtom, useLocalization } from "@hooks/use-localization";
import {
	IconFolderCode,
	IconLanguage,
	IconMenu2,
	IconRotateDot,
} from "@tabler/icons-react";
import {
	getNativeLocaleName,
	localizations,
} from "@localizations/localizations";
import { SwitchButton } from "@components/switch-button";
import { SteamCacheButton } from "./steam-cache-button";

const locales: AppLocale[] = [
	"EnUs",
	"DeDe",
	"EsEs",
	"FrFr",
	"JaJp",
	"KoKr",
	"PtPt",
	"ZhCn",
	"WaWa",
];

export function AppSettings() {
	const t = useLocalization("appDropdownMenu");
	const [settings, setSettings, resetSettings] = useAppSettings();
	const detectedLocale = useAtomValue(detectedLocaleAtom);

	const localeSelectValues = locales.map((locale) => ({
		value: locale as string,
		label: localizations[locale].meta.nativeName,
	}));

	return (
		<Menu
			closeOnItemClick={true}
			withinPortal={false}
			keepMounted={true}
			withOverlay={true}
		>
			<Menu.Target>
				<Button
					variant="filled"
					color="dark"
					ml="auto"
					fz="md"
				>
					<IconMenu2 />
				</Button>
			</Menu.Target>
			<Menu.Dropdown
				p="xs"
				bg="dark"
			>
				<Stack>
					<SwitchButton
						value={!settings.hideGameThumbnails}
						onChange={(value) => {
							setSettings({
								...settings,
								hideGameThumbnails: !value,
							});
						}}
					>
						{t("showGameThumbnails")}
					</SwitchButton>
				</Stack>
				<Menu.Item
					onClick={commands.openLogsFolder}
					leftSection={<IconFolderCode />}
				>
					{t("openLogsFolderButton")}
				</Menu.Item>
				<SteamCacheButton />
				<Tooltip
					label={t("resetRaiPalSettingsTooltip")}
					position="bottom"
				>
					<Menu.Item
						onClick={resetSettings}
						leftSection={<IconRotateDot />}
					>
						{t("resetRaiPalSettingsButton")}
					</Menu.Item>
				</Tooltip>

				<Divider my="xs" />
				<NativeSelect
					label={
						<Group>
							<span>{t("language")}</span>
							<IconLanguage />

							{/* 
							The text below is purposely left untranslated to make it easier to find,
							in case the user doesn't speak the currently selected language.
							I also included that icon that google uses for Translate,
							but I feel like nobody actually identifies that as a "language" icon. Eh.
							*/}
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
								languageName: getNativeLocaleName(detectedLocale),
							})}
						</option>
					)}
					{locales.map((locale) => (
						<option
							key={locale}
							value={locale}
						>
							{getNativeLocaleName(locale)}
						</option>
					))}
				</NativeSelect>
			</Menu.Dropdown>
		</Menu>
	);
}

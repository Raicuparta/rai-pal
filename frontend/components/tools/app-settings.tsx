import { AppLocale, commands } from "@api/bindings";
import { useAppSettings } from "@hooks/use-app-settings";
import {
	NativeSelect,
	Group,
	Box,
	Menu,
	Tabs,
	Stack,
	Tooltip,
} from "@mantine/core";
import { useAtomValue } from "jotai";
import { detectedLocaleAtom, useLocalization } from "@hooks/use-localization";
import {
	IconFolderCode,
	IconLanguage,
	IconMenu2,
	IconRotateDot,
	IconTrash,
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
			shadow="md"
		>
			<Menu.Target>
				<Tabs.Tab
					value="_" // This isn't a real tab, so random value here.
					ml="auto"
					leftSection={<IconMenu2 />}
					onClick={(e) => {
						e.stopPropagation();
					}}
				/>
			</Menu.Target>
			<Menu.Dropdown p="xs">
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
				<Tooltip
					label={t("clearRaiPalCacheTooltip")}
					position="bottom"
				>
					<Menu.Item
						onClick={commands.clearCache}
						leftSection={<IconTrash />}
					>
						{t("clearRaiPalCacheOpenModal")}
					</Menu.Item>
				</Tooltip>
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

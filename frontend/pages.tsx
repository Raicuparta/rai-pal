import { InstalledGamesPage } from "@components/installed-games/installed-games-page";
import { ModsPage } from "@components/mods/mods-page";
import { OwnedGamesPage } from "@components/owned-games/owned-games-page";
import { SettingsPage } from "@components/settings/settings-page";
import { ThanksPage } from "@components/thanks/thanks-page";
import { ThanksTabIcon } from "@components/thanks/thanks-tab-icon";
import {
	IconList,
	IconBooks,
	IconTool,
	IconSettings,
} from "@tabler/icons-react";

export const pages = {
	installedGames: {
		title: "Installed Games",
		component: InstalledGamesPage,
		icon: <IconList />,
	},
	ownedGames: {
		title: "Owned Games",
		component: InstalledGamesPage,
		icon: <IconBooks />,
	},
	mods: { title: "Mods", component: ModsPage, icon: <IconTool /> },
	settings: {
		title: "Settings",
		component: SettingsPage,
		icon: <IconSettings />,
	},
	thanks: {
		title: "Thanks",
		component: ThanksPage,
		icon: <ThanksTabIcon />,
	},
};

export type PageId = keyof typeof pages;

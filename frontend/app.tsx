import { InstalledGamesPage } from "./components/installed-games/installed-games-page";
import { ModsPage } from "./components/mods/mods-page";
import { OwnedGamesPage } from "./components/owned-games/owned-games-page";
import { SettingsPage } from "./components/settings/settings-page";
import { Tabs, Container, Stack } from "@mantine/core";
import { installedGamesAtom, ownedGamesAtom, useData } from "@hooks/use-data";
import { AppNotifications } from "@components/app-notifications";
import {
	IconBooks,
	IconHeartFilled,
	IconList,
	IconSettings,
	IconTool,
} from "@tabler/icons-react";
import { ThanksPage } from "@components/thanks/thanks-page";
import { useAtomValue } from "jotai";
import { PageTab } from "@components/page-tab";
import { useMemo } from "react";
// import { useAppUpdater } from "@hooks/use-app-updater";

const pages = {
	installedGames: {
		title: "Installed Games",
		component: InstalledGamesPage,
		icon: <IconList />,
	},
	ownedGames: {
		title: "Owned Games",
		component: OwnedGamesPage,
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
		icon: <IconHeartFilled style={{ color: "var(--mantine-color-red-9)" }} />,
	},
};

type PageId = keyof typeof pages;

type TabCounts = Record<PageId, number>;

const firstPage = Object.keys(pages)[0];

function App() {
	useData();
	const installedGames = useAtomValue(installedGamesAtom);
	const ownedGames = useAtomValue(ownedGamesAtom);

	const counts: TabCounts = useMemo(
		() => ({
			installedGames: installedGames.data.size,
			ownedGames: ownedGames.data.size,
			mods: -1,
			settings: -1,
			thanks: -1,
		}),
		[installedGames.data.size, ownedGames.data.size],
	);

	return (
		<>
			<AppNotifications />
			<Tabs
				defaultValue={firstPage}
				radius={0}
			>
				<Stack
					gap={0}
					style={{ height: "100vh" }}
				>
					<Tabs.List style={{ justifyContent: "center" }}>
						{Object.entries(pages).map(([pageId, page]) => (
							<PageTab
								key={pageId}
								id={pageId}
								page={page}
								count={counts[pageId as PageId] ?? -1}
							/>
						))}
					</Tabs.List>
					{Object.entries(pages).map(([pageId, page]) => (
						<Tabs.Panel
							key={pageId}
							style={{
								overflowY: "auto",
								flex: 1,
							}}
							value={pageId}
						>
							<Container
								h="100%"
								py="xs"
								size="lg"
							>
								<page.component />
							</Container>
						</Tabs.Panel>
					))}
				</Stack>
			</Tabs>
		</>
	);
}

export default App;

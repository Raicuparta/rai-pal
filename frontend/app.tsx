import { InstalledGamesPage } from "./components/installed-games/installed-games-page";
import { ModsPage } from "./components/mods/mods-page";
import { OwnedGamesPage } from "./components/owned-games/owned-games-page";
import { SettingsPage } from "./components/settings/settings-page";
import { Tabs, Container, Stack } from "@mantine/core";
import { useData } from "@hooks/use-data";
import { AppNotifications } from "@components/app-notifications";
import {
	IconBooks,
	IconHeartFilled,
	IconList,
	IconSettings,
	IconTool,
} from "@tabler/icons-react";
import { DonatePage } from "@components/donate/donate-page";

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
	donate: {
		title: "Donate",
		component: DonatePage,
		icon: <IconHeartFilled style={{ color: "var(--mantine-color-red-9)" }} />,
	},
};

const firstPage = Object.keys(pages)[0];

function App() {
	useData();

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
							<Tabs.Tab
								key={pageId}
								value={pageId}
								leftSection={page.icon}
							>
								{page.title}
							</Tabs.Tab>
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

import { DiscoverPage } from "@components/discover/discover-page";
import { InstalledGamesPage } from "./components/installed-games/installed-games-page";
import { ModsPage } from "./components/mods/mods-page";
import { OwnedGamesPage } from "./components/owned-games/owned-games-page";
import { SettingsPage } from "./components/settings/settings-page";
import { Tabs, Container, Stack } from "@mantine/core";

const pages = {
	installedGames: { title: "Installed Games", component: InstalledGamesPage },
	ownedGames: { title: "Owned Games", component: OwnedGamesPage },
	discover: { title: "Discover", component: DiscoverPage },
	mods: { title: "Mods", component: ModsPage },
	settings: { title: "Settings", component: SettingsPage },
};

const firstPage = Object.keys(pages)[0];

function App() {
	return (
		<Tabs
			defaultValue={firstPage}
			radius={0}
		>
			<Stack style={{ height: "100vh" }}>
				<Tabs.List style={{ justifyContent: "center" }}>
					{Object.entries(pages).map(([pageId, page]) => (
						<Tabs.Tab
							key={pageId}
							value={pageId}
						>
							{page.title}
						</Tabs.Tab>
					))}
				</Tabs.List>
				{Object.entries(pages).map(([pageId, page]) => (
					<Tabs.Panel
						key={pageId}
						style={{
							flex: 1,
						}}
						value={pageId}
					>
						<Container
							h="100%"
							pb="md"
						>
							<page.component />
						</Container>
					</Tabs.Panel>
				))}
			</Stack>
		</Tabs>
	);
}

export default App;

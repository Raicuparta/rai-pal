import { Tabs, Container, Stack } from "@mantine/core";
import { Page, PageTab } from "@components/page-tab";
import { IconBox, IconDeviceGamepad } from "@tabler/icons-react";
import { GamesPage } from "./games/games-page";
import { ModsPage } from "./mods/mods-page";
import { ThanksPage } from "./thanks/thanks-page";
import { ThanksTabIcon } from "./thanks/thanks-tab-icon";
import { useAtomValue } from "jotai";
import { gameDataAtom } from "@hooks/use-data";
import { AppSettings } from "./tools/app-settings";
import { useAppSettingSingle } from "@hooks/use-app-setting-single";
import { TabId } from "@api/bindings";

const pages: Record<TabId, Page> = {
	Games: {
		localizationKey: "games",
		component: GamesPage,
		icon: IconDeviceGamepad,
	},
	Mods: { localizationKey: "mods", component: ModsPage, icon: IconBox },
	Thanks: {
		localizationKey: "thanks",
		component: ThanksPage,
		icon: ThanksTabIcon,
	},
} as const;

export function AppTabs() {
	const gamesData = useAtomValue(gameDataAtom);

	const [selectedTab, setSelectedTab] = useAppSettingSingle("selectedTab");

	const handleTabChange = (pageId: string | null) => {
		console.log("tab changed", pageId);
		if (pageId === null || !(pageId in pages)) return;
		setSelectedTab(pageId as TabId);
	};

	const gamesCountLabel =
		gamesData.gameIds.length === Number(gamesData.totalCount)
			? `${gamesData.totalCount}`
			: `${gamesData.gameIds.length} / ${gamesData.totalCount}`;

	return (
		<Container p={0}>
			<Tabs
				value={selectedTab}
				onChange={handleTabChange}
				radius={0}
			>
				<Stack
					gap={0}
					style={{ height: "100vh" }}
				>
					<Tabs.List>
						{Object.entries(pages).map(([pageId, page]) => (
							<PageTab
								id={pageId as TabId}
								key={pageId}
								page={page}
								label={page === pages.Games ? gamesCountLabel : undefined}
							/>
						))}
						<AppSettings />
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
							>
								<page.component />
							</Container>
						</Tabs.Panel>
					))}
				</Stack>
			</Tabs>
		</Container>
	);
}

import { Tabs, Container, Stack } from "@mantine/core";
import { Page, PageTab } from "@components/page-tab";
import { useCallback } from "react";
import { usePersistedState } from "@hooks/use-persisted-state";
import { IconBox, IconDeviceGamepad, IconHammer } from "@tabler/icons-react";
import { GamesPage } from "./games/games-page";
import { ModsPage } from "./mods/mods-page";
import { ToolsPage } from "./tools/tools-page";
import { ThanksPage } from "./thanks/thanks-page";
import { ThanksTabIcon } from "./thanks/thanks-tab-icon";
import { useAtomValue } from "jotai";
import { gameDataAtom } from "@hooks/use-data";

const pages: Record<string, Page> = {
	games: {
		localizationKey: "games",
		component: GamesPage,
		icon: <IconDeviceGamepad />,
	},
	mods: { localizationKey: "mods", component: ModsPage, icon: <IconBox /> },
	tools: {
		localizationKey: "tools",
		component: ToolsPage,
		icon: <IconHammer />,
	},
	thanks: {
		localizationKey: "thanks",
		component: ThanksPage,
		icon: <ThanksTabIcon />,
	},
} as const;

const firstPage = Object.keys(pages)[0];

export function AppTabs() {
	const gamesData = useAtomValue(gameDataAtom);

	const [selectedTab, setSelectedTab] = usePersistedState(
		firstPage,
		"selected-app-tab",
	);

	const handleTabChange = useCallback(
		(pageId: string | null) => {
			if (pageId === null) return;
			setSelectedTab(pageId);
		},
		[setSelectedTab],
	);

	const gamesCountLabel =
		gamesData.gameIds.length === Number(gamesData.totalCount)
			? `${gamesData.totalCount}`
			: `${gamesData.gameIds.length} / ${gamesData.totalCount}`;

	return (
		<Tabs
			value={selectedTab}
			onChange={handleTabChange}
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
							page={page}
							label={page === pages.games ? gamesCountLabel : undefined}
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
	);
}

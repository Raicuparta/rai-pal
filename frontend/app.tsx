import { Tabs, Container, Stack } from "@mantine/core";
import { useData } from "@hooks/use-data";
import { AppNotifications } from "@components/app-notifications";
import { PageTab } from "@components/page-tab";
import { useCallback } from "react";
import { useAppUpdater } from "@hooks/use-app-updater";
import { usePersistedState } from "@hooks/use-persisted-state";
import { PageId, pages } from "./pages";
import { useDataCounts } from "@hooks/use-data-counts";

const firstPage = Object.keys(pages)[0];

function App() {
	useAppUpdater();
	useData();
	const counts = useDataCounts();

	const [selectedTab, setSelectedTab] = usePersistedState(
		firstPage,
		"selected-app-tab",
	);

	const handleTabChange = useCallback(
		(pageId: string | null) => {
			setSelectedTab(pageId as PageId);
		},
		[setSelectedTab],
	);

	return (
		<>
			<AppNotifications />
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

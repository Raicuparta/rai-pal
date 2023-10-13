import { Box, Flex, Paper, Stack, Text } from "@mantine/core";
import { VirtuosoGrid } from "react-virtuoso";
import styles from "./discover.module.css";
import { useMemo, useState } from "react";
import { RefreshButton } from "@components/refresh-button";
import { GameEngineBrand } from "@api/bindings";
import { EngineSelect } from "@components/engine-select";
import { DiscoverGameCard } from "./discover-game-card";
import { useAppStore } from "@hooks/use-app-state";

export function DiscoverPage() {
	const discoverGames = useAppStore((state) => state.remoteState.discoverGames);

	const [engine, setEngine] = useState<GameEngineBrand>();

	const filteredGames = useMemo(
		() =>
			(engine
				? discoverGames?.filter((game) => game.engine == engine)
				: discoverGames) ?? [],
		[discoverGames, engine],
	);

	return (
		<Stack h="100%">
			<Flex
				align="center"
				gap="md"
			>
				<Text style={{ flex: 1 }}>
					{filteredGames.length} games you don&apos;t own
				</Text>
				<EngineSelect
					onChange={setEngine}
					value={engine}
				/>
				<RefreshButton />
			</Flex>
			<Paper
				h="100%"
				className={styles.wrapper}
			>
				<VirtuosoGrid
					overscan={200}
					data={filteredGames}
					listClassName={styles.listContainer}
					components={{
						Header: () => <Box className={styles.spacer} />,
						Footer: () => <Box className={styles.spacer} />,
					}}
					itemContent={(_, game) => <DiscoverGameCard game={game} />}
				/>
			</Paper>
		</Stack>
	);
}

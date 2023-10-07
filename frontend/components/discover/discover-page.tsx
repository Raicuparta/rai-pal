import { useUnownedGames } from "@hooks/use-backend-data";
import { Box, Flex, Paper, Stack, Text } from "@mantine/core";
import { VirtuosoGrid } from "react-virtuoso";
import styles from "./discover.module.css";
import { useMemo, useState } from "react";
import { RefreshButton } from "@components/refresh-button";
import { ErrorPopover } from "@components/error-popover";
import { GameEngineBrand } from "@api/bindings";
import { EngineSelect } from "@components/engine-select";
import { steamCommands } from "../../util/steam";

export function DiscoverPage() {
	const [unownedGames, isLoading, refresh, error, clearError] =
		useUnownedGames();

	const [engine, setEngine] = useState<GameEngineBrand>();

	const filteredGames = useMemo(
		() =>
			engine
				? unownedGames.filter((game) => game.engine == engine)
				: unownedGames,
		[unownedGames, engine],
	);

	return (
		<Stack h="100%">
			<Flex
				justify="space-between"
				align="center"
				gap="md"
			>
				<EngineSelect
					onChange={setEngine}
					value={engine}
				/>
				<Text>{filteredGames.length} games</Text>
				<ErrorPopover
					error={error}
					clearError={clearError}
				>
					<RefreshButton
						loading={isLoading}
						onClick={refresh}
					/>
				</ErrorPopover>
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
					itemContent={(index) => (
						<img
							className={styles.item}
							onClick={() =>
								steamCommands.openStorePage(filteredGames[index].id)
							}
							height={87}
							width={231}
							src={`https://cdn.cloudflare.steamstatic.com/steam/apps/${filteredGames[index].id}/capsule_231x87.jpg`}
						/>
					)}
				/>
			</Paper>
		</Stack>
	);
}

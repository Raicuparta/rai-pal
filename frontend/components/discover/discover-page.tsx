import { useUnownedGames } from "@hooks/use-backend-data";
import { Box, Paper } from "@mantine/core";
import { VirtuosoGrid } from "react-virtuoso";
import styles from "./discover.module.css";
import { shell } from "@tauri-apps/api";

export function DiscoverPage() {
	const [unownedGames, isLoading, refresh, error, clearError] =
		useUnownedGames();

	return (
		<Paper
			h="100%"
			className={styles.wrapper}
		>
			<VirtuosoGrid
				overscan={200}
				data={unownedGames}
				itemClassName={styles.itemContainer}
				listClassName={styles.listContainer}
				components={{
					Header: () => <Box className={styles.spacer} />,
					Footer: () => <Box className={styles.spacer} />,
				}}
				itemContent={(index) => (
					<img
						className={styles.item}
						onClick={() =>
							shell.open(
								`https://steampowered.com/app/${unownedGames[index].id}`,
							)
						}
						height={87}
						width={231}
						src={`https://cdn.cloudflare.steamstatic.com/steam/apps/${unownedGames[index].id}/capsule_231x87.jpg`}
					/>
				)}
			/>
		</Paper>
	);
}

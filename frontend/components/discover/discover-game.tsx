import { DiscoverGame } from "@api/bindings";
import { steamCommands } from "../../util/steam";
import styles from "./discover.module.css";
import { Box } from "@mantine/core";
import { useState } from "react";

type Props = {
	readonly game: DiscoverGame;
};

export function DiscoverGameCard(props: Props) {
	const [isNsfwRevealed, setIsNsfwRevealed] = useState<boolean>(false);

	const isHidden = !isNsfwRevealed && props.game.nsfw;

	return (
		<Box className={styles.game}>
			{isHidden && (
				<div className={styles.nsfwLabel}>
					<span>NSFW</span>
					<span>Click to reveal</span>
				</div>
			)}
			<img
				className={isHidden ? styles.nsfwImage : undefined}
				onClick={() =>
					isHidden
						? setIsNsfwRevealed(true)
						: steamCommands.openStorePage(props.game.id)
				}
				height={87}
				width={231}
				src={`https://cdn.cloudflare.steamstatic.com/steam/apps/${props.game.id}/capsule_231x87.jpg`}
			/>
		</Box>
	);
}

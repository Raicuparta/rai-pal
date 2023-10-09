import { DiscoverGame } from "@api/bindings";
import { steamCommands } from "../../util/steam";
import styles from "./discover.module.css";
import { Box, Loader, Overlay } from "@mantine/core";
import { useState } from "react";

type Props = {
	readonly game: DiscoverGame;
};

export function DiscoverGameCard(props: Props) {
	const [isNsfwRevealed, setIsNsfwRevealed] = useState<boolean>(false);
	const [isLoaded, setIsLoaded] = useState<boolean>(false);
	const [isBroken, setIsBroken] = useState<boolean>(false);

	const isHidden = !isNsfwRevealed && props.game.nsfw;

	return (
		<Box className={styles.game}>
			{isLoaded && isHidden && (
				<div className={styles.nsfwLabel}>
					<span>NSFW</span>
					<span>Click to reveal</span>
				</div>
			)}

			{!isBroken && !isLoaded && (
				<Overlay className={styles.loadingImage}>
					<Loader
						color="gray"
						size="xl"
					/>
				</Overlay>
			)}
			<img
				className={isHidden ? styles.nsfwImage : undefined}
				onClick={() =>
					isHidden
						? setIsNsfwRevealed(true)
						: steamCommands.openStorePage(props.game.id)
				}
				onError={() => setIsBroken(true)}
				onLoad={() => setIsLoaded(true)}
				height={87}
				width={231}
				src={
					isBroken
						? "images/fallback-thumbnail.png"
						: `https://cdn.cloudflare.steamstatic.com/steam/apps/${props.game.id}/capsule_231x87.jpg`
				}
			/>
		</Box>
	);
}

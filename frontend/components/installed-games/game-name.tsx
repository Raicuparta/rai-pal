import { InstalledGame } from "@api/bindings";
import { Code, Flex } from "@mantine/core";
import styles from "./installed-games.module.css";

type Props = {
	readonly game: InstalledGame;
};

export function GameName(props: Props) {
	return (
		<Flex className={styles.gameName}>
			{props.game.name}
			{props.game.discriminator && (
				<Code opacity={0.5}>{props.game.discriminator}</Code>
			)}
		</Flex>
	);
}

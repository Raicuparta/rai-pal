import { InstalledGame } from "@api/bindings";
import { Code, Flex } from "@mantine/core";

type Props = {
	readonly game: InstalledGame;
};

export function GameName(props: Props) {
	return (
		<Flex
			gap="xs"
			wrap="wrap"
		>
			{props.game.name}
			{props.game.discriminator && (
				<Code opacity={0.5}>{props.game.discriminator}</Code>
			)}
		</Flex>
	);
}

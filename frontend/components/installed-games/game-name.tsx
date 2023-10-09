import { Game } from "@api/bindings";
import { Code, Flex } from "@mantine/core";

type Props = {
	readonly game: Game;
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

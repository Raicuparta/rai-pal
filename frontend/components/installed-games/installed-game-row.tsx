import { Table, Flex } from "@mantine/core";
import { Game } from "@api/bindings";
import { GameName } from "./game-name";
import { GameThumbnail } from "@components/game-thumbnail";
import {
	ArchitectureBadge,
	EngineBadge,
	// OperatingSystemBadge,
	UnityBackendBadge,
} from "@components/color-coded-badge";

export function InstalledGameRow(_: number, game: Game) {
	return (
		<>
			<GameThumbnail url={game.thumbnailUrl} />
			<Table.Td>
				<GameName game={game} />
			</Table.Td>
			{/* <Table.Td>
				<OperatingSystemBadge value={game.operatingSystem} />
			</Table.Td> */}
			<Table.Td>
				<ArchitectureBadge value={game.architecture} />
			</Table.Td>
			<Table.Td>
				<UnityBackendBadge value={game.scriptingBackend} />
			</Table.Td>
			<Table.Td>
				<Flex
					align="center"
					gap="xs"
				>
					<EngineBadge
						maw={70}
						value={game.engine?.brand}
						label={game.engine?.version?.display}
					/>
				</Flex>
			</Table.Td>
		</>
	);
}

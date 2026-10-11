import { Group, Table } from "@mantine/core";
import { GameMod } from "@api/bindings";
import { ModDetails } from "./mod-details";
import { ModActions } from "./mod-actions";
import { compareReleaseDates } from "../../date-formatter";

type Props = {
	readonly mods: Record<string, GameMod>;
};

export function ModsTable(props: Props) {
	return (
		<Table>
			<Table.Tbody>
				{Object.values(props.mods)
					.sort((a, b) =>
						compareReleaseDates(
							a.download?.releaseDate,
							b.download?.releaseDate,
						),
					)
					.map((mod) => (
						<Table.Tr key={mod.id}>
							<Table.Td>
								<ModDetails
									mod={mod}
									versionText={mod.download?.id}
								/>
							</Table.Td>
							<Table.Td maw={200}>
								<Group justify="right">
									<ModActions mod={mod} />
								</Group>
							</Table.Td>
						</Table.Tr>
					))}
			</Table.Tbody>
		</Table>
	);
}

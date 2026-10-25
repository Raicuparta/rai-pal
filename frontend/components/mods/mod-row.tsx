import { Group, Table } from "@mantine/core";
import { GameMod } from "@api/bindings";
import { ReactNode } from "react";
import { ModDetails } from "./mod-details";
import { ModActions } from "./mod-actions";

type Props = {
	readonly mod: GameMod;
	readonly leading?: ReactNode;
};

export function ModRow({ mod, leading }: Props) {
	return (
		<Table.Tr>
			<Table.Td>
				<ModDetails
					mod={mod}
					versionText={mod.download?.id}
					leading={leading}
				/>
			</Table.Td>
			<Table.Td maw={200}>
				<Group justify="right">
					<ModActions mod={mod} />
				</Group>
			</Table.Td>
		</Table.Tr>
	);
}

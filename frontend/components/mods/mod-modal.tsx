import { Box, Stack } from "@mantine/core";
import { GameMod } from "@api/bindings";
import { DebugData } from "@components/debug-data";
import { TableContainer } from "@components/table/table-container";
import { useMemo } from "react";
import { ModsTable } from "./mods-table";
import { SubPage } from "@components/sub-page";

type Props = {
	readonly mod: GameMod;
	readonly onClose: () => void;
};

export function ModModal(props: Props) {
	const wrappedMod = useMemo(() => [props.mod], [props.mod]);

	return (
		<SubPage onClose={props.onClose}>
			<Box>
				<TableContainer>
					<ModsTable
						mods={wrappedMod}
						onClick={props.onClose}
					/>
				</TableContainer>
			</Box>
			<Stack
				p="xs"
				gap="xl"
			>
				<Stack>
					<DebugData data={props.mod} />
				</Stack>
			</Stack>
		</SubPage>
	);
}

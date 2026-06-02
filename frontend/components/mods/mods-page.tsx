import { Card, Group, Stack } from "@mantine/core";
import { useMemo, useState } from "react";
import { RefreshButton } from "@components/refresh-button";
import { ModModal } from "./mod-modal";
import { ModsTable } from "./mods-table";
import { TableContainer } from "@components/table/table-container";
import { useAtomValue } from "jotai";
import { modsAtom } from "@hooks/use-data";

export function ModsPage() {
	const [selectedModId, setSelectedId] = useState<string>();

	const mods = useAtomValue(modsAtom);

	const selectedMod = useMemo(() => {
		const result = selectedModId ? mods[selectedModId] : undefined;

		return result;
	}, [selectedModId, mods]);

	return (
		<Stack h="100%">
			{selectedMod && (
				<ModModal
					onClose={() => setSelectedId(undefined)}
					mod={selectedMod}
				/>
			)}
			{!selectedMod && (
				<>
					<Group justify="end">
						<RefreshButton />
					</Group>
					<Card
						p={0}
						flex={1}
						bg="dark"
					>
						<TableContainer style={{ overflowY: "scroll" }}>
							<ModsTable
								mods={mods}
								onClick={(mod) => setSelectedId(mod.id)}
							/>
						</TableContainer>
					</Card>
				</>
			)}
		</Stack>
	);
}

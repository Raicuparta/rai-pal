import { Button, Group, Stack, Table, Text } from "@mantine/core";
import { useMemo, useState } from "react";
import { TableContainer } from "@components/table/table-container";
import { RefreshButton } from "@components/refresh-button";
import { openModsFolder } from "@api/bindings";
import { IconFolderCog } from "@tabler/icons-react";
import {
	EngineBadge,
	UnityBackendBadge,
} from "@components/badges/color-coded-badge";
import { useAtomValue } from "jotai";
import { modsAtom } from "@hooks/use-data";
import { ModModal } from "./mod-modal";

export function ModsPage() {
	const [selectedModId, setSelectedId] = useState<string>();

	const mods = useAtomValue(modsAtom);

	const selectedMod = useMemo(() => {
		const result = selectedModId ? mods[selectedModId] : undefined;

		return result;
	}, [selectedModId, mods]);

	return (
		<Stack h="100%">
			{selectedMod ? (
				<ModModal
					onClose={() => setSelectedId(undefined)}
					mod={selectedMod}
				/>
			) : null}
			<Group justify="end">
				<Button
					onClick={openModsFolder}
					leftSection={<IconFolderCog />}
				>
					Open Mods Folder
				</Button>
				<RefreshButton />
			</Group>
			<TableContainer>
				<Table highlightOnHover>
					<Table.Thead>
						<Table.Tr>
							<Table.Th>Mod</Table.Th>
							<Table.Th>Current</Table.Th>
							<Table.Th>Latest</Table.Th>
							<Table.Th
								ta="center"
								w={100}
							>
								Loader
							</Table.Th>
							<Table.Th
								w={100}
								ta="center"
							>
								Engine
							</Table.Th>
							<Table.Th
								w={100}
								ta="center"
							>
								Backend
							</Table.Th>
						</Table.Tr>
					</Table.Thead>
					<Table.Tbody>
						{Object.entries(mods).map(([modId, mod]) => (
							<Table.Tr
								key={modId}
								onClick={() => setSelectedId(mod.common.id)}
							>
								<Table.Td ta="left">
									<Text>{mod.remoteMod?.title ?? modId}</Text>
									{mod.remoteMod?.description && (
										<Text
											size="sm"
											opacity={0.5}
										>
											{mod.remoteMod.description}
										</Text>
									)}
								</Table.Td>
								<Table.Td>
									{mod.localMod?.manifest?.version ?? "Unknown"}
								</Table.Td>
								<Table.Td>
									{mod.remoteMod?.downloads[0]?.version ?? "Unknown"}
								</Table.Td>
								<Table.Td>TODO</Table.Td>
								<Table.Td>
									<EngineBadge value={mod.common.engine} />
								</Table.Td>
								<Table.Td>
									<UnityBackendBadge value={mod.common.unityBackend} />
								</Table.Td>
							</Table.Tr>
						))}
					</Table.Tbody>
				</Table>
			</TableContainer>
		</Stack>
	);
}

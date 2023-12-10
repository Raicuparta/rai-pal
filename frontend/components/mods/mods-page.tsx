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
import { ModModal } from "./mod-modal";
import { useUnifiedMods } from "@hooks/use-unified-mods";
import { DebugData } from "@components/debug-data";
import { useAtomValue } from "jotai";
import { modLoadersAtom } from "@hooks/use-data";

export function ModsPage() {
	const [selectedModId, setSelectedId] = useState<string>();

	const mods = useUnifiedMods();
	const modLoaders = useAtomValue(modLoadersAtom);

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
									<Text>{mod.remote?.title ?? modId}</Text>
									{mod.remote?.description && (
										<Text
											size="sm"
											opacity={0.5}
										>
											{mod.remote.description}
										</Text>
									)}
								</Table.Td>
								<Table.Td>{mod.local?.manifest?.version ?? "Unknown"}</Table.Td>
								<Table.Td>
									{mod.remote?.downloads[0]?.version ?? "Unknown"}
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

			<DebugData data={modLoaders} />
		</Stack>
	);
}

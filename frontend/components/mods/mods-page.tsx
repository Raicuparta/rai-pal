import { Button, Group, Stack, Table, Text } from "@mantine/core";
import { Fragment, useMemo, useState } from "react";
import { TableContainer } from "@components/table/table-container";
import { RefreshButton } from "@components/refresh-button";
import { openModsFolder } from "@api/bindings";
import { IconFolderCog } from "@tabler/icons-react";
import {
	EngineBadge,
	UnityBackendBadge,
} from "@components/badges/color-coded-badge";
import { useAtomValue } from "jotai";
import { modLoadersAtom } from "@hooks/use-data";
import { ModModal } from "./mod-modal";

const defaultSelection = ["", ""];

export function ModsPage() {
	const [[selectedModLoaderId, selectedModId], setSelectedId] =
		useState(defaultSelection);

	const modLoaders = useAtomValue(modLoadersAtom);

	const selectedMod = useMemo(() => {
		const result = selectedModId
			? modLoaders[selectedModLoaderId]?.mods[selectedModId]
			: undefined;

		return result;
	}, [modLoaders, selectedModLoaderId, selectedModId]);

	return (
		<Stack h="100%">
			{selectedMod ? (
				<ModModal
					onClose={() => setSelectedId(defaultSelection)}
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
						{Object.values(modLoaders).map((modLoader) => (
							<Fragment key={modLoader.id}>
								{Object.entries(modLoader.mods).map(([modId, mod]) => (
									<Table.Tr
										key={modId}
										onClick={() => setSelectedId([modLoader.id, mod.common.id])}
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
										<Table.Td>{modLoader.id}</Table.Td>
										<Table.Td>
											<EngineBadge value={mod.common.engine} />
										</Table.Td>
										<Table.Td>
											<UnityBackendBadge value={mod.common.unityBackend} />
										</Table.Td>
									</Table.Tr>
								))}
							</Fragment>
						))}
					</Table.Tbody>
				</Table>
			</TableContainer>
		</Stack>
	);
}

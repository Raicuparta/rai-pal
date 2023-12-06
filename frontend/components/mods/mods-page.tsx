import { Button, Group, Stack, Table } from "@mantine/core";
import { Fragment } from "react";
import { TableContainer } from "@components/table/table-container";
import { RefreshButton } from "@components/refresh-button";
import { openModFolder, openModsFolder } from "@api/bindings";
import { IconFolderCog } from "@tabler/icons-react";
import {
	EngineBadge,
	UnityBackendBadge,
} from "@components/badges/color-coded-badge";
import { useAtomValue } from "jotai";
import { modLoadersAtom } from "@hooks/use-data";

export function ModsPage() {
	const modLoaders = useAtomValue(modLoadersAtom);

	return (
		<Stack h="100%">
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
							<Table.Th>Source</Table.Th>
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
										onClick={() => openModFolder(modLoader.id, modId)}
									>
										<Table.Td ta="left">
											{modId} ({mod.remoteMod?.title})
										</Table.Td>
										<Table.Td ta="left">
											{mod.remoteMod?.sourceCode ?? mod.localMod?.path}
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

import { Button, Group, Stack, Table } from "@mantine/core";
import { Fragment, useState } from "react";
import { TableContainer } from "@components/table/table-container";
import { RefreshButton } from "@components/refresh-button";
import { GameMod, openModsFolder } from "@api/bindings";
import { IconFolderCog } from "@tabler/icons-react";
import {
	EngineBadge,
	UnityBackendBadge,
} from "@components/badges/color-coded-badge";
import { useAtomValue } from "jotai";
import { modLoadersAtom } from "@hooks/use-data";
import { ModModal } from "./mod-modal";

export function ModsPage() {
	const [selectedMod, setSelectedMod] = useState<GameMod>();
	const modLoaders = useAtomValue(modLoadersAtom);

	return (
		<Stack h="100%">
			{selectedMod ? (
				<ModModal
					onClose={() => setSelectedMod(undefined)}
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
										onClick={() => setSelectedMod(mod)}
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

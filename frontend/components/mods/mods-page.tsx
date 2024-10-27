import { Button, Group, Stack, Table, Text } from "@mantine/core";
import { useMemo, useState } from "react";
import { TableContainer } from "@components/table/table-container";
import { RefreshButton } from "@components/refresh-button";
import { commands } from "@api/bindings";
import { IconFolderCog } from "@tabler/icons-react";
import {
	EngineBadge,
	UnityBackendBadge,
} from "@components/badges/color-coded-badge";
import { ModModal } from "./mod-modal";
import { UnifiedMod, useUnifiedMods } from "@hooks/use-unified-mods";
import { ModVersionBadge } from "./mod-version-badge";
import { ItemName } from "@components/item-name";
import { getModTitle } from "@util/game-mod";
import { DeprecatedBadge } from "./deprecated-badge";

export function ModsPage() {
	const [selectedModId, setSelectedId] = useState<string>();

	const mods = useUnifiedMods();
	const filteredMods = useMemo(() => {
		const result: Record<string, UnifiedMod> = {};
		for (const [modId, mod] of Object.entries(mods)) {
			if (!mod.local && mod.remote?.deprecated) {
				continue;
			}

			result[modId] = mod;
		}
		return result;
	}, [mods]);

	const selectedMod = useMemo(() => {
		const result = selectedModId ? filteredMods[selectedModId] : undefined;

		return result;
	}, [selectedModId, filteredMods]);

	return (
		<Stack>
			{selectedMod ? (
				<ModModal
					onClose={() => setSelectedId(undefined)}
					mod={selectedMod}
				/>
			) : null}
			<Group justify="end">
				<Button
					onClick={commands.openModsFolder}
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
							<Table.Th ta="center">Version</Table.Th>
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
						{Object.entries(filteredMods).map(([modId, mod]) => (
							<Table.Tr
								key={modId}
								onClick={() => setSelectedId(mod.common.id)}
							>
								<Table.Td>
									{mod.remote?.deprecated && <DeprecatedBadge />}
									<ItemName
										label={
											mod.remote?.author
												? `by ${mod.remote?.author}`
												: undefined
										}
									>
										{getModTitle(mod)}
									</ItemName>
									{mod.remote?.description && (
										<Text
											size="sm"
											opacity={0.5}
										>
											{mod.remote.description}
										</Text>
									)}
								</Table.Td>
								<Table.Td>
									<ModVersionBadge
										localVersion={mod.local?.manifest?.version}
										remoteVersion={mod.remote?.latestVersion?.id}
									/>
								</Table.Td>
								<Table.Td ta="center">{mod.common.loaderId}</Table.Td>
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

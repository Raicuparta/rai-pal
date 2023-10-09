import { Badge, Button, Flex, Stack, Table } from "@mantine/core";
import { Fragment } from "react";
import { useModLoaders } from "@hooks/use-backend-data";
import { TableContainer } from "@components/table/table-container";
import { engineColor } from "../../util/color";
import { RefreshButton } from "@components/refresh-button";
import { ErrorPopover } from "@components/error-popover";
import { openModFolder, openModsFolder } from "@api/bindings";
import { IconFolderCog } from "@tabler/icons-react";
import { UnityBackendBadge } from "@components/badges/unity-backend.badge";

export function ModsPage() {
	const [modLoaders, isLoading, refreshMods, error, clearError] =
		useModLoaders();

	return (
		<Stack h="100%">
			<Flex
				justify="end"
				gap="md"
			>
				<Button
					onClick={openModsFolder}
					leftSection={<IconFolderCog />}
				>
					Open Mods Folder
				</Button>
				<ErrorPopover
					error={error}
					clearError={clearError}
				>
					<RefreshButton
						loading={isLoading}
						onClick={refreshMods}
					/>
				</ErrorPopover>
			</Flex>
			<TableContainer>
				<Table highlightOnHover>
					<Table.Thead>
						<Table.Tr>
							<Table.Th>Mod</Table.Th>
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
								{modLoader.mods.map((mod) => (
									<Table.Tr
										key={mod.path}
										onClick={() => openModFolder(modLoader.id, mod.id)}
									>
										<Table.Td ta="left">{mod.name}</Table.Td>
										<Table.Td>{modLoader.id}</Table.Td>
										<Table.Td>
											<Badge
												color={mod.engine ? engineColor[mod.engine] : "dark"}
											>
												{mod.engine ?? "Unknown"}
											</Badge>
										</Table.Td>
										<Table.Td>
											<UnityBackendBadge backend={mod.scriptingBackend} />
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

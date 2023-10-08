import { Badge, Flex, Stack, Table } from "@mantine/core";
import { Fragment } from "react";
import { useModLoaders } from "@hooks/use-backend-data";
import { TableContainer } from "@components/table/table-container";
import { engineColor, scriptingBackendColor } from "../../util/color";
import { RefreshButton } from "@components/refresh-button";
import { ErrorPopover } from "@components/error-popover";
import { openModFolder } from "@api/bindings";

export function ModsPage() {
	const [modLoaders, isLoading, refreshMods, error, clearError] =
		useModLoaders();

	return (
		<Stack h="100%">
			<Flex
				justify="end"
				gap="md"
			>
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
							<Table.Th w={100}>Loader</Table.Th>
							<Table.Th w={100}>Engine</Table.Th>
							<Table.Th w={100}>Backend</Table.Th>
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
										<Table.Td>{mod.name}</Table.Td>
										<Table.Td>{modLoader.id}</Table.Td>
										<Table.Td>
											<Badge
												color={mod.engine ? engineColor[mod.engine] : "dark"}
											>
												{mod.engine ?? "Unknown"}
											</Badge>
										</Table.Td>
										<Table.Td>
											<Badge
												color={
													mod.scriptingBackend
														? scriptingBackendColor[mod.scriptingBackend]
														: "dark"
												}
												fullWidth
											>
												{mod.scriptingBackend ?? "-"}
											</Badge>
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

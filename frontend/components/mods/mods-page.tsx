import { Badge, Flex, Stack, Table } from "@mantine/core";
import { Fragment } from "react";
import { useModLoaders } from "@hooks/use-backend-data";
import { openModFolder } from "@api/bindings";
import { TableContainer } from "@components/table/table-container";
import { scriptingBackendColor } from "../../util/color";
import { RefreshButton } from "@components/refresh-button";

export function ModsPage() {
	const [modLoaders, isLoading, refreshMods] = useModLoaders();

	return (
		<Stack h="100%">
			<Flex justify="end">
				<RefreshButton
					loading={isLoading}
					onClick={refreshMods}
				/>
			</Flex>
			<TableContainer>
				<Table highlightOnHover>
					<Table.Thead>
						<Table.Tr>
							<Table.Th>Mod</Table.Th>
							<Table.Th w={100}>Loader</Table.Th>
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
												color={scriptingBackendColor[mod.scriptingBackend]}
												fullWidth
											>
												{mod.scriptingBackend}
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

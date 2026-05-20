import { Table, Text } from "@mantine/core";
import { TableContainer } from "@components/table/table-container";
import { UnifiedMod } from "@hooks/use-unified-mods";
import { ModVersionBadge } from "./mod-version-badge";
import { ItemName } from "@components/item-name";
import { getModTitle } from "@util/game-mod";
import { DeprecatedBadge } from "./deprecated-badge";
import { useLocalization } from "@hooks/use-localization";

type Props = {
	mods: UnifiedMod[];
	onClick?: (mod: UnifiedMod) => void;
};

export function ModsTable(props: Props) {
	const t = useLocalization("modsPage");

	return (
		<Table highlightOnHover={Boolean(props.onClick)}>
			<Table.Thead pos="sticky">
				<Table.Tr>
					<Table.Th>{t("tableColumnMod")}</Table.Th>
					<Table.Th ta="center">{t("tableColumnVersion")}</Table.Th>
					<Table.Th
						w={100}
						ta="center"
					>
						{t("tableColumnGameEngine")}
					</Table.Th>
					<Table.Th
						w={100}
						ta="center"
					>
						{t("tableColumnUnityBackend")}
					</Table.Th>
				</Table.Tr>
			</Table.Thead>
			<Table.Tbody>
				{Object.entries(props.mods).map(([modId, mod]) => (
					<Table.Tr
						key={modId}
						onClick={
							props.onClick
								? () => props.onClick && props.onClick(mod)
								: undefined
						}
					>
						<Table.Td>
							{mod.remote?.deprecated && <DeprecatedBadge />}
							<ItemName
								label={
									mod.remote?.author
										? `${t("modByAuthor", { authorName: mod.remote?.author })}`
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
								local={mod.local}
								remote={mod.remote}
							/>
						</Table.Td>
						<Table.Td>{mod.merged.engine}</Table.Td>
						<Table.Td>{mod.merged.unityBackend}</Table.Td>
					</Table.Tr>
				))}
			</Table.Tbody>
		</Table>
	);
}

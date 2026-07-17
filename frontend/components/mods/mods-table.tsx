import { Badge, Group, Table, Text } from "@mantine/core";
import { DeprecatedBadge } from "./deprecated-badge";
import { useLocalization } from "@hooks/use-localization";
import { GameMod } from "@api/bindings";

type Props = {
	readonly mods: Record<string, GameMod>;
	readonly onClick?: (mod: GameMod) => void;
};

export function ModsTable(props: Props) {
	const { t } = useLocalization("modsPage");

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
							{mod.deprecated && <DeprecatedBadge />}
							<Group gap="xs">
								<span>{mod.title}</span>
								{mod.author && (
									<Text
										size="xs"
										opacity={0.5}
									>{`${t("modByAuthor", { authorName: mod.author })}`}</Text>
								)}
							</Group>
							{mod.description && (
								<Text
									size="sm"
									opacity={0.5}
								>
									{mod.description}
								</Text>
							)}
						</Table.Td>
						<Table.Td ta="center">
							<Badge color="gray">{mod.download?.id ?? "-"}</Badge>
						</Table.Td>
						<Table.Td>{mod.engine}</Table.Td>
						<Table.Td>{mod.unityBackend}</Table.Td>
					</Table.Tr>
				))}
			</Table.Tbody>
		</Table>
	);
}

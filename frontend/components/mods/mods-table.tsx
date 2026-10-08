import { Badge, Group, Table, Text } from "@mantine/core";
import { DeprecatedBadge } from "./deprecated-badge";
import { useLocalization } from "@hooks/use-localization";
import { GameMod } from "@api/bindings";

type Props = {
	readonly mods: Record<string, GameMod>;
	readonly onClick?: (mod: GameMod) => void;
};

const dateFormatter = Intl.DateTimeFormat("default", {
	year: "numeric",
	month: "long",
	day: "2-digit",
});

export function ModsTable(props: Props) {
	const { t } = useLocalization("modsPage");

	return (
		<Table highlightOnHover={Boolean(props.onClick)}>
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
							{mod.download?.releaseDate && (
								<Text
									size="xs"
									opacity={0.5}
								>
									{dateFormatter.format(new Date(mod.download.releaseDate))}
								</Text>
							)}
						</Table.Td>
						<Table.Td
							w={100}
							ta="center"
						>
							{mod.engine}
							{mod.unityBackend && (
								<Text
									size="xs"
									opacity={0.5}
								>
									{mod.unityBackend}
								</Text>
							)}
						</Table.Td>
					</Table.Tr>
				))}
			</Table.Tbody>
		</Table>
	);
}

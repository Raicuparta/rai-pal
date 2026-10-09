import { Badge, Group, Table, Text } from "@mantine/core";
import { DeprecatedBadge } from "./deprecated-badge";
import { useLocalization } from "@hooks/use-localization";
import { GameMod } from "@api/bindings";
import { dateFormatter, compareReleaseDates } from "../../date-formatter";

type Props = {
	readonly mods: Record<string, GameMod>;
	readonly onClick?: (mod: GameMod) => void;
};

export function ModsTable(props: Props) {
	const { t } = useLocalization("modsPage");

	return (
		<Table highlightOnHover={Boolean(props.onClick)}>
			<Table.Tbody>
				{Object.values(props.mods)
					.sort((a, b) =>
						compareReleaseDates(
							a.download?.releaseDate,
							b.download?.releaseDate,
						),
					)
					.map((mod) => (
						<Table.Tr
							key={mod.id}
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

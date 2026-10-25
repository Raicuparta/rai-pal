import { Group, Stack, Table, Text, ThemeIcon } from "@mantine/core";
import { GameMod, ModFamily } from "@api/bindings";
import { IconChevronDown, IconChevronRight } from "@tabler/icons-react";
import { useDisclosure } from "@mantine/hooks";
import { ModRow } from "./mod-row";

type Props = {
	readonly family: ModFamily;
	readonly mods: GameMod[];
};

export function ModFamilyRow({ family, mods }: Props) {
	const [expanded, { toggle }] = useDisclosure(false);

	return (
		<>
			<Table.Tr
				onClick={toggle}
				data-hover="true"
			>
				<Table.Td colSpan={2}>
					<Group
						wrap="nowrap"
						align="center"
					>
						<ThemeIcon
							color="gray"
							size="sm"
						>
							{expanded ? <IconChevronDown /> : <IconChevronRight />}
						</ThemeIcon>
						<Stack gap={0}>
							<Text
								fw="bold"
								size="lg"
							>
								{family.title}
							</Text>
							<Text
								fz="sm"
								opacity={0.75}
							>
								{family.description}
							</Text>
						</Stack>
					</Group>
				</Table.Td>
			</Table.Tr>
			{expanded &&
				mods.map((mod) => (
					<ModRow
						key={mod.id}
						mod={mod}
						leading={
							<ThemeIcon
								variant="transparent"
								size="sm"
							/>
						}
					/>
				))}
		</>
	);
}

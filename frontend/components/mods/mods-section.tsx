import { Divider, Group, Stack, Table, Tooltip } from "@mantine/core";
import { IconInfoCircle } from "@tabler/icons-react";
import { ReactNode } from "react";

type Props = {
	readonly label: string;
	readonly description?: string;
	readonly children: ReactNode;
};

export function ModsSection({ label, description, children }: Props) {
	return (
		<Stack gap={0}>
			<Divider
				label={
					<Group gap={4}>
						{label}
						{description && (
							<Tooltip label={description}>
								<IconInfoCircle />
							</Tooltip>
						)}
					</Group>
				}
			/>
			<Table>
				<Table.Tbody>{children}</Table.Tbody>
			</Table>
		</Stack>
	);
}

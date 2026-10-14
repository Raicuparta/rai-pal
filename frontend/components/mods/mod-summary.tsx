import { Group, Stack, Text } from "@mantine/core";
import { ReactNode } from "react";

type Props = {
	readonly title: string;
	readonly description: string;
	readonly leading?: ReactNode;
	readonly titleAdornment?: ReactNode;
	readonly details?: ReactNode;
	readonly trailing?: ReactNode;
};

export function ModSummary({
	title,
	description,
	leading,
	titleAdornment,
	details,
	trailing,
}: Props) {
	return (
		<Group
			wrap="nowrap"
			align="center"
		>
			{leading}
			<Stack gap={0}>
				<Group>
					<Text
						fw="bold"
						size="md"
						c="var(--mantine-color-bright)"
					>
						{title}
					</Text>
					{titleAdornment}
				</Group>
				{details}
				<Text
					fz="sm"
					opacity={0.75}
				>
					{description}
				</Text>
			</Stack>
			{trailing}
		</Group>
	);
}

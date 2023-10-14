import { Button, Divider, Stack, StackProps } from "@mantine/core";

interface Props extends StackProps {
	readonly label: string;
	readonly children: React.ReactNode;
}

export function CommandButtonGroup({ label, children, ...props }: Props) {
	return (
		<Stack
			gap="xs"
			{...props}
		>
			<Divider label={label} />
			<Button.Group
				orientation="vertical"
				w="fit-content"
			>
				{children}
			</Button.Group>
		</Stack>
	);
}

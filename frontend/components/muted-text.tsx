import { Text } from "@mantine/core";

type Props = {
	readonly children: React.ReactNode;
};

export function MutedText(props: Props) {
	return (
		<Text
			component="span"
			size="xs"
			opacity={0.5}
		>
			{props.children}
		</Text>
	);
}

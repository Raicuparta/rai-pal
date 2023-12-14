import { Text } from "@mantine/core";

type Props = {
	readonly children: React.ReactNode;
};

export function MutedText(props: Props) {
	return (
		<Text
			size="sm"
			opacity={0.5}
		>
			{props.children}
		</Text>
	);
}

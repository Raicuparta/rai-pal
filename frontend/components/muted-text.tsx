import { Text, TextProps } from "@mantine/core";

interface Props extends TextProps {
	readonly children: React.ReactNode;
}

export function MutedText(props: Props) {
	return (
		<Text
			component="span"
			size="xs"
			opacity={0.5}
			{...props}
		/>
	);
}

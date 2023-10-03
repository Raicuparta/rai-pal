import { BackgroundImage, Table } from "@mantine/core";

type Props = {
	readonly url: string | null;
};

export function GameThumbnail(props: Props) {
	return (
		<BackgroundImage
			src={props.url ?? ""}
			component={Table.Td}
		>
			{"\u200b"}
		</BackgroundImage>
	);
}

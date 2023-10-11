import { Box, Image } from "@mantine/core";

type Props = {
	readonly src?: string | null;
};

export function ModalImage(props: Props) {
	return (
		<Box>
			<Image
				fallbackSrc="images/fallback-thumbnail.png"
				bg="dark"
				radius="md"
				src={props.src}
				fit="contain"
				height={87}
			/>
		</Box>
	);
}

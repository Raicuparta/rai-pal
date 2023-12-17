import { Image } from "@mantine/core";

type Props = {
	readonly src?: string | null;
};

export function ModalImage(props: Props) {
	return (
		props.src && (
			<Image
				fallbackSrc="images/fallback-thumbnail.png"
				bg="dark"
				radius="md"
				src={props.src}
				fit="contain"
				height={50}
			/>
		)
	);
}

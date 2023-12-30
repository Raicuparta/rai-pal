import { Image } from "@mantine/core";

type Props = {
	readonly src: string;
};

export function ModalImage(props: Props) {
	return (
		<Image
			fallbackSrc="images/thumbnails/Manual.png"
			bg="dark"
			radius="md"
			src={props.src}
			fit="contain"
			height={50}
		/>
	);
}

import { Image, ImageProps } from "@mantine/core";

export function GameImage({ src, ...props }: ImageProps) {
	return (
		<Image
			fallbackSrc="images/fallback-thumbnail.png"
			src={src || "images/fallback-thumbnail.png"}
			fit="contain"
			{...props}
		/>
	);
}

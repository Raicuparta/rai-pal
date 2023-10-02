import styles from "./game-thumbnail.module.css";
import { BackgroundImage } from "@mantine/core";

type Props = {
	readonly url: string;
};

export function GameThumbnail(props: Props) {
	return (
		<BackgroundImage
			src={props.url}
			className={styles.wrapper}
		/>
	);
}

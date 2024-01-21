import { Button, ButtonProps } from "@mantine/core";
import styles from "./donate.module.css";

interface Props extends ButtonProps {
	readonly href: string;
}

export function DonateLinkButton(props: Props) {
	return (
		<Button
			component="a"
			target="_blank"
			color="blue"
			variant="light"
			className={styles.donateButton}
			{...props}
		/>
	);
}

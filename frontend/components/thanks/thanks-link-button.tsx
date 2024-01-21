import { Button, ButtonProps } from "@mantine/core";
import styles from "./thanks.module.css";

interface Props extends ButtonProps {
	readonly href: string;
}

export function ThanksLinkButton(props: Props) {
	return (
		<Button
			component="a"
			target="_blank"
			color="blue"
			variant="light"
			className={styles.thanksButton}
			{...props}
		/>
	);
}

import { Button, ButtonProps } from "@mantine/core";
import styles from "./thanks.module.css";
import { commands } from "@api/bindings";

interface Props extends ButtonProps {
	readonly href: string;
}

function handleClick(event: React.MouseEvent<HTMLAnchorElement, MouseEvent>) {
	event.preventDefault();
	// TODO: check the open_url command on the backend,
	// there's a comment there explaining why this is needed.
	commands.openUrl(event.currentTarget.href);
}

export function ThanksLinkButton(props: Props) {
	return (
		<Button
			component="a"
			target="_blank"
			color="blue"
			variant="light"
			onClick={handleClick}
			className={styles.thanksButton}
			{...props}
		/>
	);
}

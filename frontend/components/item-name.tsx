import { Code, Flex } from "@mantine/core";
import styles from "./components.module.css";

type Props = {
	readonly children: React.ReactNode;
	readonly label?: string | null;
};

export function ItemName(props: Props) {
	return (
		<Flex className={styles.gameName}>
			{props.children}
			{props.label && <Code opacity={0.5}>{props.label}</Code>}
		</Flex>
	);
}

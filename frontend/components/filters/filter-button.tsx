import { Button, Checkbox, Tooltip } from "@mantine/core";
import styles from "./filters.module.css";

type Props = {
	readonly isVisible: boolean;
	readonly onClick: () => void;
	readonly filterOption?: string;
	readonly note?: string;
};

export function FilterButton(props: Props) {
	return (
		props.filterOption && (
			<Tooltip
				label={props.note}
				disabled={!props.note}
			>
				<Button
					fullWidth
					justify="start"
					leftSection={
						<Checkbox
							tabIndex={-1}
							readOnly
							className={styles.checkbox}
							checked={props.isVisible}
						/>
					}
					onClick={props.onClick}
				>
					{props.filterOption}
					{props.note && " *"}
				</Button>
			</Tooltip>
		)
	);
}

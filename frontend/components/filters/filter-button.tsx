import { Button, Checkbox, Tooltip } from "@mantine/core";
import styles from "./filters.module.css";

type Props = {
	readonly isVisible: boolean;
	readonly isUnavailable: boolean;
	readonly onClick: () => void;
	readonly filterOption: string;
};

export function FilterButton(props: Props) {
	return (
		<Tooltip
			label="Not implemented"
			disabled={!props.isUnavailable}
		>
			<Button
				disabled={props.isUnavailable}
				fullWidth
				justify="start"
				leftSection={
					<Checkbox
						disabled={props.isUnavailable}
						tabIndex={-1}
						readOnly
						className={styles.checkbox}
						checked={!props.isUnavailable && props.isVisible}
					/>
				}
				onClick={props.onClick}
			>
				{props.filterOption}
			</Button>
		</Tooltip>
	);
}

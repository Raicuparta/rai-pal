import { Button, Checkbox, Tooltip } from "@mantine/core";
import { useCallback } from "react";
import styles from "./filters.module.css";

type Props<TFilterOption extends string> = {
	readonly isHidden: boolean;
	readonly isUnavailable: boolean;
	readonly onClick: (value: TFilterOption | null) => void;
	readonly filterOption: TFilterOption | null;
};

export function FilterButton<TFilterOption extends string>(
	props: Props<TFilterOption>,
) {
	const handleClick = useCallback(() => {
		props.onClick(props.filterOption);
	}, [props]);

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
						checked={!props.isUnavailable && !props.isHidden}
					/>
				}
				onClick={handleClick}
			>
				{props.filterOption ?? "Unknown"}
			</Button>
		</Tooltip>
	);
}

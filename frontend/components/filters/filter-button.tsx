import { FilterOption } from "@components/table/table-head";
import { Button, Checkbox, Tooltip } from "@mantine/core";
import { useCallback } from "react";
import { UnknownFilterOption } from "./filter-select";
import styles from "./filters.module.css";

type Props<TFilterOption extends string> = {
	readonly isHidden: boolean;
	readonly isUnavailable: boolean;
	readonly onClick: (value: TFilterOption | null) => void;
	readonly filterOption: UnknownFilterOption | FilterOption<TFilterOption>;
};

export function FilterButton<TFilterOption extends string>(
	props: Props<TFilterOption>,
) {
	const handleClick = useCallback(() => {
		props.onClick(props.filterOption.value);
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
				{props.filterOption.label || props.filterOption.value}
			</Button>
		</Tooltip>
	);
}

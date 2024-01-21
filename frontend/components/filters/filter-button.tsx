import { FilterOption } from "@components/table/table-head";
import { Button, Checkbox } from "@mantine/core";
import { useCallback } from "react";
import { UnknownFilterOption } from "./filter-select";
import styles from "./filters.module.css";

type Props<TFilterOption extends string> = {
	readonly isHidden: boolean;
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
		<Button
			fullWidth
			justify="start"
			leftSection={
				<Checkbox
					className={styles.checkbox}
					checked={!props.isHidden}
				/>
			}
			onClick={handleClick}
		>
			{props.filterOption.label || props.filterOption.value}
		</Button>
	);
}

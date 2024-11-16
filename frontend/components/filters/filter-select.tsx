import { Button, Group, Stack, Tooltip } from "@mantine/core";
import { TableColumn } from "../table/table-head";
import { IconEye, IconEyeClosed, IconRestore } from "@tabler/icons-react";
import { useCallback, useMemo } from "react";
import { FilterButton } from "./filter-button";

type Props<TFilterOption extends string> = {
	readonly id: string;
	readonly filterOptions: Record<TFilterOption, boolean>;
	readonly onClick: (id: string, value: TFilterOption | null) => void;
};

export function FilterSelect<TFilterOption extends string>({
	id,
	filterOptions,
	onClick,
}: Props<TFilterOption>) {
	const sortedEntries = useMemo(
		() => Object.entries(filterOptions).sort(([a], [b]) => a.localeCompare(b)),
		[filterOptions],
	);

	return (
		<Stack gap="xs">
			{filterOptions && (
				<>
					<Button.Group orientation="vertical">
						{sortedEntries.map(([filterOption, isSelected]) => (
							<FilterButton
								filterOption={filterOption}
								onClick={(value) => onClick(id, filterOption as TFilterOption)}
								isHidden={isSelected as boolean}
								// isUnavailable={Boolean(
								// 	filterOption &&
								// 		column.unavailableValues?.includes(filterOption),
								// )}
								isUnavailable={false}
								key={filterOption}
							/>
						))}
					</Button.Group>
					{/* <Button
						onClick={handleReset}
						leftSection={<IconRestore fontSize={10} />}
						disabled={(hiddenValues?.length || 0) === 0}
					>
						Reset
					</Button> */}
				</>
			)}
		</Stack>
	);
}

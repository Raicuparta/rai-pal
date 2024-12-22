import { Button, Stack } from "@mantine/core";
import { useMemo } from "react";
import { FilterButton } from "./filter-button";
import { GamesFilter } from "@api/bindings";
import { useDataQuery } from "@hooks/use-data-query";

type Props = {
	readonly id: keyof GamesFilter;
	readonly onClick: (
		id: keyof GamesFilter,
		value: keyof GamesFilter[typeof id],
	) => void;
};

export function FilterSelect({ id, onClick }: Props) {
	const [dataQuery] = useDataQuery();
	const sortedEntries = useMemo(
		() =>
			Object.entries(dataQuery.filter[id]).sort(([a], [b]) =>
				a.localeCompare(b),
			),
		[dataQuery.filter, id],
	);

	return (
		<Stack gap="xs">
			{dataQuery.filter && (
				<>
					<Button.Group orientation="vertical">
						<Button disabled>{id}</Button>
						{sortedEntries.map(([filterOption, isSelected]) => (
							<FilterButton
								filterOption={filterOption}
								onClick={() =>
									onClick(id, filterOption as keyof GamesFilter[typeof id])
								}
								isHidden={!isSelected as boolean}
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

import { Button, Stack } from "@mantine/core";
import { FilterButton } from "./filter-button";

type Props = {
	readonly id: string;
	readonly possibleValues: (string | null)[];
	readonly currentValues: (string | null)[];
	readonly onClick: (id: string, value: string | null) => void;
};

export function FilterSelect({
	id,
	possibleValues,
	currentValues,
	onClick,
}: Props) {
	return (
		<Stack gap="xs">
			<Button.Group orientation="vertical">
				<Button disabled>{id}</Button>
				{possibleValues.map((possibleValue) => (
					<FilterButton
						filterOption={possibleValue ?? "Unknown"}
						onClick={() => onClick(id, possibleValue)}
						isVisible={currentValues.includes(possibleValue)}
						// isUnavailable={Boolean(
						// 	filterOption &&
						// 		column.unavailableValues?.includes(filterOption),
						// )}
						isUnavailable={false} // TODO isUnavailable
						key={possibleValue}
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
		</Stack>
	);
}

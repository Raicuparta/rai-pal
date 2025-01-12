import { Button, Stack } from "@mantine/core";
import { FilterButton } from "./filter-button";
import { GamesFilter } from "@api/bindings";

export type FilterKey = keyof GamesFilter;
export type FilterValue<TFilterKey extends FilterKey> =
	GamesFilter[TFilterKey][number];

type Props<TFilterKey extends keyof GamesFilter> = {
	readonly id: TFilterKey;
	readonly possibleValues: Array<FilterValue<TFilterKey>>;
	readonly currentValues: Array<FilterValue<TFilterKey>>;
	readonly onClick: (id: TFilterKey, value: FilterValue<TFilterKey>) => void;
};

type FilterDetails<TKey extends keyof GamesFilter> = {
	title: string;

	// Text that shows for each filter type for the "empty value" option.
	// If not defined, the empty option is hidden from the filter menu.
	emptyOption?: string;

	valueNotes?: Partial<Record<NonNullable<GamesFilter[TKey][number]>, string>>;
};

const filterDetails: { [key in keyof GamesFilter]: FilterDetails<key> } = {
	architectures: {
		title: "Architecture",
		emptyOption: "Unknown",
	},
	engines: {
		title: "Engine",
		emptyOption: "Unknown",
		valueNotes: {
			Godot: "Rai Pal doesn't fully support Godot yet.",
			GameMaker: "Rai Pal doesn't fully support GameMaker yet.",
		},
	},
	unityScriptingBackends: {
		title: "Unity Backend",
		emptyOption: "Unknown",
	},
	tags: {
		title: "Tags",
		emptyOption: "Untagged",
	},
	installed: {
		title: "Status",
	},
	providers: {
		title: "Provider",
		valueNotes: {
			Xbox: "Xbox PC games only show on Rai Pal once they're installed.",
		},
	},
};

export function FilterSelect<TFilterKey extends FilterKey>({
	id,
	possibleValues,
	currentValues,
	onClick,
}: Props<TFilterKey>) {
	return (
		<Stack gap="xs">
			<Button.Group orientation="vertical">
				<Button disabled>{filterDetails[id].title}</Button>
				{possibleValues.map((possibleValue) => (
					<FilterButton
						filterOption={possibleValue ?? filterDetails[id].emptyOption}
						onClick={() => onClick(id, possibleValue)}
						isVisible={currentValues.includes(possibleValue)}
						note={filterDetails[id].valueNotes?.[possibleValue]}
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
			{/* TODO: reset button */}
		</Stack>
	);
}

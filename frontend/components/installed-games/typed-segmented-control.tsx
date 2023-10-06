import { SegmentedControl, SegmentedControlItem } from "@mantine/core";

export type SegmentedControlData<T extends string> = SegmentedControlItem & {
	value: T | "";
};

export type TypedSegmentedControlProps<T extends string> = {
	readonly data: SegmentedControlData<T>[];
	readonly onChange: (value?: T) => void;
	readonly value?: T;
};

export function TypedSegmentedControl<T extends string>(
	props: TypedSegmentedControlProps<T>,
) {
	return (
		<SegmentedControl
			color={props.value ? "violet" : undefined}
			data={props.data}
			onChange={(value) => props.onChange((value as T) || undefined)}
			value={props.value || ""}
		/>
	);
}

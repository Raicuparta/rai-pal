import { SegmentedControl, SegmentedControlItem } from "@mantine/core";

export type SegmentedControlData<T extends string> = SegmentedControlItem & {
  value: T | "";
};

type Props<T extends string> = {
  readonly data: SegmentedControlData<T>[];
  readonly onChange: (value?: T) => void;
  readonly value?: T;
};

export function TypedSegmentedControl<T extends string>(props: Props<T>) {
  return (
    <SegmentedControl
      color={props.value ? "violet" : undefined}
      onChange={(value) => props.onChange((value as T) || undefined)}
      value={props.value || ""}
      data={props.data}
    />
  );
}

import { SegmentedControl, SegmentedControlItem } from "@mantine/core";

export type SegmentedControlData<T extends string> = SegmentedControlItem & {
  value: T | "";
};

type Props<T extends string> = {
  data: SegmentedControlData<T>[];
  onChange: (value?: T) => void;
  value?: T;
};

export function TypedSegmentedControl<T extends string>(props: Props<T>) {
  return (
    <SegmentedControl
      onChange={(value) => props.onChange((value as T) || undefined)}
      value={props.value || ""}
      data={props.data}
    />
  );
}

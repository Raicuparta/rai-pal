import { Button, Indicator, Popover } from "@mantine/core";
import { MdFilterAlt } from "react-icons/md";

type Props = {
  readonly children: React.ReactNode;
  readonly active: boolean;
};

export function FilterMenu(props: Props) {
  return (
    <Popover>
      <Popover.Target>
        <Indicator offset={8} disabled={!props.active}>
          <Button variant="default" leftSection={<MdFilterAlt />}>
            Filter
          </Button>
        </Indicator>
      </Popover.Target>
      <Popover.Dropdown>{props.children}</Popover.Dropdown>
    </Popover>
  );
}

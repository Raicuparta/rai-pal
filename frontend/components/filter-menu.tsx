import { Button, Popover } from "@mantine/core";
import { MdFilterAlt } from "react-icons/md";

type Props = {
  readonly children: React.ReactNode;
};

export function FilterMenu(props: Props) {
  return (
    <Popover>
      <Popover.Target>
        <Button variant="default" leftSection={<MdFilterAlt />}>
          Filter
        </Button>
      </Popover.Target>
      <Popover.Dropdown>{props.children}</Popover.Dropdown>
    </Popover>
  );
}

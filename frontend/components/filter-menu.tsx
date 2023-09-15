import { Button, Popover } from "@mantine/core";
import { MdFilterAlt } from "react-icons/md";

type Props = {
  children: React.ReactNode;
};

export const FilterMenu = (props: Props) => (
  <Popover>
    <Popover.Target>
      <Button variant="default" leftIcon={<MdFilterAlt />}>
        Filter
      </Button>
    </Popover.Target>
    <Popover.Dropdown>{props.children}</Popover.Dropdown>
  </Popover>
);

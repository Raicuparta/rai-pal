import { Button } from "@mantine/core";
import { MdFilterAltOff } from "react-icons/md";

type Props = {
  readonly setFilter: (filter: undefined) => void;
};

export const ResetButton = (props: Props) => (
  <Button
    leftSection={<MdFilterAltOff />}
    onClick={() => props.setFilter(undefined)}
  >
    Reset
  </Button>
);

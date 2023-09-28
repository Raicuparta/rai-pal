import { Button } from "@mantine/core";
import { MdRefresh } from "react-icons/md";

type Props = {
  readonly onClick: () => void;
  readonly loading: boolean;
};

export const RefreshButton = (props: Props) => (
  <Button
    onClick={props.onClick}
    loading={props.loading}
    style={{ flex: 1, maxWidth: 200 }}
    leftSection={<MdRefresh />}
  >
    {props.loading ? "Loading..." : "Refresh"}
  </Button>
);

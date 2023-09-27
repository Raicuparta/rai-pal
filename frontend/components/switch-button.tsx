import { Button, ButtonProps, Switch } from "@mantine/core";

interface Props extends ButtonProps {
  value: boolean;
  onChange: (value: boolean) => void;
}

export const SwitchButton = ({ value, onChange, ...props }: Props) => (
  <Button
    variant="filled"
    color="dark"
    onClick={() => onChange(!value)}
    justify="start"
    leftSection={<Switch style={{ pointerEvents: "none" }} checked={value} />}
    {...props}
  />
);

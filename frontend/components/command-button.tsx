import { useAsyncCommand } from "@hooks/use-async-command";
import {
  Box,
  Button,
  ButtonProps,
  CloseButton,
  Code,
  Flex,
  Popover,
  Stack,
} from "@mantine/core";
import { MdError } from "react-icons/md";

interface Props<TResult> extends ButtonProps {
  onClick: () => Promise<TResult>;
  icon: React.ReactNode;
}

export function CommandButton<TResult>({
  onClick,
  icon,
  ...props
}: Props<TResult>) {
  const [executeCommand, isLoading, success, error, clearError] =
    useAsyncCommand(onClick);

  return (
    <>
      <Popover opened={Boolean(error)} position="bottom" width={400}>
        <Popover.Target>
          <Button
            justify="start"
            variant={success || error ? "filled" : "default"}
            color={error ? "red" : "green"}
            loading={isLoading}
            {...props}
            onClick={executeCommand}
            leftSection={icon}
          />
        </Popover.Target>
        <Popover.Dropdown>
          <Stack>
            <Flex justify="space-between">
              <Box component={MdError} c="red" />
              <CloseButton onClick={clearError} />
            </Flex>
            <Code style={{ overflow: "auto", flex: 1 }}>
              <pre>{error}</pre>
            </Code>
          </Stack>
        </Popover.Dropdown>
      </Popover>
    </>
  );
}

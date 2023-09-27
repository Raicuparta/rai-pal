import { useAsyncCommand } from "@hooks/use-async-command";
import {
  Box,
  Button,
  ButtonProps,
  CloseButton,
  Flex,
  Popover,
} from "@mantine/core";
import { MdCheck, MdError } from "react-icons/md";

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
      <Popover variant="" opened={Boolean(error)} position="bottom" width={400}>
        <Popover.Target>
          <Button
            disabled={Boolean(error)}
            variant="default"
            loading={isLoading}
            {...props}
            onClick={executeCommand}
            leftSection={success ? <Box component={MdCheck} c="green" /> : icon}
          />
        </Popover.Target>
        <Popover.Dropdown>
          <Flex justify="space-between">
            <Box component={MdError} c="red" />
            <CloseButton onClick={clearError} />
          </Flex>
          <Box component="code" style={{ overflow: "auto", flex: 1 }}>
            {error}
          </Box>
        </Popover.Dropdown>
      </Popover>
    </>
  );
}

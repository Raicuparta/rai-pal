import {
  Button,
  Card,
  CardBody,
  CardHeader,
  Divider,
  Menu,
  MenuButton,
  MenuItem,
  MenuList,
  Stack,
  Text,
} from "@chakra-ui/react";
import { ChevronDownIcon, TriangleUpIcon } from "@chakra-ui/icons";
import { SteamApp } from "./steam-app";

type Props = {
  steamApp: SteamApp;
};

export const GameCard = (props: Props) => {
  return (
    <Card size="sm">
      <CardHeader>
        <Text>{props.steamApp.name}</Text>
      </CardHeader>
      <Divider />
      <CardBody>
        <Stack direction={"row"}>
          <Button
            leftIcon={<TriangleUpIcon transform={"rotate(90deg)"} />}
            size="sm"
          >
            Start Game
          </Button>
          <Menu>
            <MenuButton size="sm" as={Button} rightIcon={<ChevronDownIcon />}>
              Install mod
            </MenuButton>
            <MenuList>
              <MenuItem>Unity Explorer</MenuItem>
              <MenuItem>The other one</MenuItem>
            </MenuList>
          </Menu>
        </Stack>
      </CardBody>
    </Card>
  );
};

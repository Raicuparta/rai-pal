import { useState } from "react";
import { invoke } from "@tauri-apps/api/tauri";
import {
  Button,
  Card,
  CardBody,
  CardHeader,
  CircularProgress,
  Code,
  Container,
  Divider,
  Menu,
  MenuButton,
  MenuItem,
  MenuList,
  Select,
  Stack,
  Tab,
  TabList,
  TabPanel,
  TabPanels,
  Tabs,
  Text,
  useColorMode,
} from "@chakra-ui/react";
import { ChevronDownIcon } from "@chakra-ui/icons";

type AppLaunch = {
  executable?: string;
  app_type?: string;
  os_list?: string;
};
type AppDetails = {
  name: string;
  launch_map: Record<string, AppLaunch>;
  install_path: String;
};
type AppDetailsMap = Record<number, AppDetails>;

function App() {
  const [steamApps, setSteamApps] = useState<AppDetailsMap>();
  const [isLoading, setIsLoading] = useState(false);
  const { colorMode, toggleColorMode } = useColorMode();

  async function greet() {
    setIsLoading(true);
    // Learn more about Tauri commands at https://tauri.app/v1/guides/features/command
    const json: string = await invoke("get_steam_apps_json");

    console.log(json);
    setSteamApps(JSON.parse(json));
    setIsLoading(false);
  }

  return (
    <Tabs isFitted display="flex" flexDirection={"column"} height="100vh">
      <TabList>
        <Container maxWidth="2xl" display="flex" flexDirection="row">
          <Tab>Home</Tab>
          <Tab>Steam Games</Tab>
          <Tab>Other Games</Tab>
          <Tab>Settings</Tab>
        </Container>
      </TabList>
      <TabPanels flex="1" overflow={"scroll"}>
        <Container maxWidth="2xl">
          <TabPanel>
            <Stack gap={2}>
              <Button
                type="submit"
                onClick={() => {
                  toggleColorMode();
                  greet();
                }}
                isLoading={isLoading}
                loadingText="Looking and finding and searching..."
              >
                Get games from Steam
              </Button>
              {steamApps &&
                Object.entries(steamApps).map(([appId, steamApp]) => (
                  <Card size="sm" key={appId}>
                    <CardHeader>
                      <Text>{steamApp.name}</Text>
                    </CardHeader>
                    <Divider />
                    <CardBody>
                      <Menu>
                        <MenuButton as={Button} rightIcon={<ChevronDownIcon />}>
                          Install mod
                        </MenuButton>
                        <MenuList>
                          <MenuItem>Unity Explorer</MenuItem>
                          <MenuItem>The other one</MenuItem>
                        </MenuList>
                      </Menu>
                    </CardBody>
                  </Card>
                ))}
            </Stack>
          </TabPanel>
        </Container>
      </TabPanels>
    </Tabs>
  );
}

export default App;

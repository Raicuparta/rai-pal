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
  Stack,
  Tab,
  TabList,
  TabPanel,
  TabPanels,
  Tabs,
  Text,
  useColorMode,
} from "@chakra-ui/react";

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
        <Container display="flex" flexDirection="row">
          <Tab>Home</Tab>
          <Tab>Steam Games</Tab>
          <Tab>Other Games</Tab>
          <Tab>Settings</Tab>
        </Container>
      </TabList>
      <TabPanels flex="1" overflow={"scroll"}>
        <Container>
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
                  <Card size="sm">
                    <CardHeader>{steamApp.name}</CardHeader>
                    <Divider />
                    <CardBody>
                      <Code
                        whiteSpace={"nowrap"}
                        width={"full"}
                        overflow={"scroll"}
                      >
                        {steamApp.install_path}
                      </Code>
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

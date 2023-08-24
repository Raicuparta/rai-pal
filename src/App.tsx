import { useState } from "react";
import { invoke } from "@tauri-apps/api/tauri";
import {
  Button,
  Card,
  CardBody,
  CardHeader,
  Code,
  Container,
  Divider,
  Stack,
  Tab,
  TabList,
  TabPanel,
  TabPanels,
  Tabs,
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
  const [name, setName] = useState("");
  const { colorMode, toggleColorMode } = useColorMode();

  async function greet() {
    // Learn more about Tauri commands at https://tauri.app/v1/guides/features/command
    setSteamApps(JSON.parse(await invoke("greet", { name })));
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
            <Button
              type="submit"
              onClick={() => {
                toggleColorMode();
                greet();
              }}
            >
              Greet {colorMode}
            </Button>
            <Stack gap={2}>
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

import { useState } from "react";
import { invoke } from "@tauri-apps/api/tauri";
import {
  Button,
  Container,
  Stack,
  Tab,
  TabList,
  TabPanel,
  TabPanels,
  Tabs,
} from "@chakra-ui/react";
import { AppMap } from "./steam-app";
import { GameCard } from "./game-card";

function App() {
  const [steamApps, setSteamApps] = useState<AppMap>();
  const [isLoading, setIsLoading] = useState(false);

  async function greet() {
    setIsLoading(true);
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
      <TabPanels flex="1" overflow={"auto"}>
        <Container maxWidth="2xl">
          <TabPanel>
            <Stack>
              <Button
                type="submit"
                onClick={greet}
                isLoading={isLoading}
                loadingText="Looking and finding and searching..."
              >
                Get games from Steam
              </Button>
              {steamApps &&
                Object.entries(steamApps).map(([appId, steamApp]) => (
                  <GameCard key={appId} steamApp={steamApp} />
                ))}
            </Stack>
          </TabPanel>
        </Container>
      </TabPanels>
    </Tabs>
  );
}

export default App;

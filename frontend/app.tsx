import { InstalledGamesPage } from "./components/installed-games/installed-games-page";
import { ModsPage } from "./components/mods/mods-page";
import { OwnedGamesPage } from "./components/owned-games/owned-games-page";
import { SettingsPage } from "./components/settings/settings-page";
import { Tabs, Container, Stack } from "@mantine/core";

import "./app.css";

const pages = {
  installedGames: { title: "Installed Games", component: InstalledGamesPage },
  ownedGames: { title: "Owned Games", component: OwnedGamesPage },
  mods: { title: "Mods", component: ModsPage },
  settings: { title: "Settings", component: SettingsPage },
};

const firstPage = Object.keys(pages)[0];

function App() {
  return (
    <Tabs defaultValue={firstPage} radius={0}>
      <Stack style={{ height: "100vh" }}>
        <Tabs.List style={{ justifyContent: "center" }}>
          {Object.entries(pages).map(([pageId, page]) => (
            <Tabs.Tab key={pageId} value={pageId}>
              {page.title}
            </Tabs.Tab>
          ))}
        </Tabs.List>
        {Object.entries(pages).map(([pageId, page]) => (
          <Tabs.Panel
            key={pageId}
            value={pageId}
            style={{
              flex: 1,
              overflow: "auto",
            }}
          >
            <Container pb="md" h="100%">
              <page.component />
            </Container>
          </Tabs.Panel>
        ))}
      </Stack>
    </Tabs>
  );
}

export default App;

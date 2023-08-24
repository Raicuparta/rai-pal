import { Button, Stack } from "@chakra-ui/react";
import { AppMap } from "../steam-app";
import { GameCard } from "../game-card";
import { useState } from "react";
import { invoke } from "@tauri-apps/api";

export const HomePage = () => {
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
  );
};

import { Button, Flex } from "@mantine/core";
import { MdCheckCircle, MdInstallDesktop, MdOpenInNew } from "react-icons/md";
import { OwnedUnityGame } from "@api/game/steam-owned-unity-games";
import { open } from "@tauri-apps/api/shell";

export function OwnedGameRow(_: number, ownedUnityGame: OwnedUnityGame) {
  return (
    <>
      <td>{ownedUnityGame.name}</td>
      <td>{ownedUnityGame.installed ? <MdCheckCircle /> : ""}</td>
      <td>
        <Flex gap="md">
          <Button
            compact
            radius="xl"
            onClick={() =>
              open(`https://steampowered.com/app/${ownedUnityGame.id}`)
            }
          >
            <MdOpenInNew />
          </Button>
          <Button
            onClick={() => open(`steam://install/${ownedUnityGame.id}`)}
            compact
            radius="xl"
          >
            <MdInstallDesktop />
          </Button>
        </Flex>
      </td>
    </>
  );
}

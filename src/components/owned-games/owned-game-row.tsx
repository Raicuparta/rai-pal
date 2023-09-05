import { Button } from "@mantine/core";
import { MdCheckCircle, MdOpenInNew } from "react-icons/md";
import { OwnedUnityGame } from "@api/game/steam-owned-unity-games";

export function OwnedGameRow(_: number, ownedUnityGame: OwnedUnityGame) {
  return (
    <>
      <td>{ownedUnityGame.name}</td>
      <td>{ownedUnityGame.installed ? <MdCheckCircle /> : ""}</td>
      <td>
        <Button
          compact
          radius="xl"
          // onClick={() => openSteamGame(ownedUnityGame.id)}
        >
          <MdOpenInNew />
        </Button>
      </td>
    </>
  );
}

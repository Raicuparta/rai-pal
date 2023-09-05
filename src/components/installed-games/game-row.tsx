import { SteamGame } from "@api/game/steam-game";
import { GameExecutableRow } from "./game-executable-row";

type Props = Readonly<{
  steamApp: SteamGame;
  index: number;
}>;

export function GameRow(props: Props) {
  return props.steamApp.distinctExecutables.map((executable) => (
    <tr key={executable.name + executable.steamLaunchId}>
      <GameExecutableRow game={props.steamApp} executable={executable} />
    </tr>
  ));
}

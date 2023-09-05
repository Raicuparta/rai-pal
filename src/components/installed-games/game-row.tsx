import { Game } from "@api/game/game";
import { GameExecutableRow } from "./game-executable-row";

type Props = Readonly<{
  steamApp: Game;
  index: number;
}>;

export function GameRow(props: Props) {
  return props.steamApp.distinctExecutables?.map((executable) => (
    <tr key={executable.id}>
      <GameExecutableRow game={props.steamApp} executable={executable} />
    </tr>
  ));
}

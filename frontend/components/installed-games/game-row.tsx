import { Game } from "@api/bindings";
import { GameExecutableRow } from "./game-executable-row";

type Props = Readonly<{
  game: Game;
  index: number;
}>;

export function GameRow(props: Props) {
  return props.game.distinctExecutables?.map((executable) => (
    <tr key={executable.id}>
      <GameExecutableRow game={props.game} executable={executable} />
    </tr>
  ));
}

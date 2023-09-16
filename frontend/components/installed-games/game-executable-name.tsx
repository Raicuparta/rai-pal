import { Game } from "@api/bindings";
import { Code } from "@mantine/core";

type Props = {
  readonly game: Game;
};

export function GameName(props: Props) {
  return <>
    {props.game.name}{" "}
    {props.game.discriminator ? <Code opacity={0.5}>{props.game.discriminator}</Code> : null}
  </>
}

import { Game } from "@api/bindings";
import { Code } from "@mantine/core";

type Props = {
  game: Game;
};

export const GameName = (props: Props) => (
  <>
    {props.game.name}{" "}
    {props.game.discriminator && (
      <Code opacity={0.5}>{props.game.discriminator}</Code>
    )}
  </>
);

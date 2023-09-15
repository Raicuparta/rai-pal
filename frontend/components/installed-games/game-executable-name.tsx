import { GameData } from "./installed-game-row";
import { Code } from "@mantine/core";

type Props = {
  data: GameData;
};

export const GameName = (props: Props) => (
  <>
    {props.data.executable.name}{" "}
    {props.data.executable.discriminator && (
      <Code opacity={0.5}>{props.data.executable.discriminator}</Code>
    )}
  </>
);

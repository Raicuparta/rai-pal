import { Game } from "@api/bindings";
import { GameData } from "./installed-game-row";
import { Code } from "@mantine/core";

type Props = {
  data: GameData;
};

const getInnerSuffix = (executable: Game, allExecutables: Game[]) => {
  const steamLaunch = executable.steamLaunch;
  if (!steamLaunch) return executable.name;

  // We only use the description to discriminate between executables if all descriptions are unique.
  const uniqueDescriptions = new Set(
    allExecutables.map((executable) => executable.steamLaunch?.description)
  );
  if (uniqueDescriptions.size !== allExecutables.length) {
    return [executable.name, executable.steamLaunch?.arguments]
      .filter(Boolean)
      .join(" ");
  }

  return steamLaunch.description ?? "";
};

// Suffix used to discriminate between multiple game executables of the same game.
export const getGameNameSuffix = (data: GameData) => {
  return data.executable.nameSuffix;

  // TODO move this to backend.
  // const executables = Object.values(data.game.executables);

  // // If there's only one executable, nothing to discriminate against.
  // if (executables.length <= 1) return "";

  // const innerSuffix = getInnerSuffix(data.executable, executables);
  // if (!innerSuffix) return "";

  // return `(${innerSuffix})`;
};

export const GameName = (props: Props) => {
  const suffix = getGameNameSuffix(props.data);

  return (
    <>
      {props.data.executable.name}{" "}
      {suffix && <Code opacity={0.5}>{suffix}</Code>}
    </>
  );
};

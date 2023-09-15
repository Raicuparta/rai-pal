import { GameExecutable } from "@api/bindings";
import { GameExecutableData } from "./installed-game-row";
import { Code } from "@mantine/core";

type Props = {
  data: GameExecutableData;
};

const getInnerSuffix = (
  executable: GameExecutable,
  allExecutables: GameExecutable[]
) => {
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
export const getGameExecutableNameSuffix = (data: GameExecutableData) => {
  return data.executable.nameSuffix;

  // TODO move this to backend.
  // const executables = Object.values(data.game.executables);

  // // If there's only one executable, nothing to discriminate against.
  // if (executables.length <= 1) return "";

  // const innerSuffix = getInnerSuffix(data.executable, executables);
  // if (!innerSuffix) return "";

  // return `(${innerSuffix})`;
};

export const GameExecutableName = (props: Props) => {
  const suffix = getGameExecutableNameSuffix(props.data);

  return (
    <>
      {props.data.executable.name}{" "}
      {suffix && <Code opacity={0.5}>{suffix}</Code>}
    </>
  );
};

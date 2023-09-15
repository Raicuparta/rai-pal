import { Button, Code, Modal, Stack } from "@mantine/core";
import { GameExecutableData } from "./installed-game-row";
import { useModLoaders } from "@hooks/use-backend-data";
import {
  installMod,
  openGameFolder,
  openGameModsFolder,
  startGame,
} from "@api/bindings";
import { Fragment } from "react";
import { GameExecutableName } from "./game-executable-name";

type Props = {
  data: GameExecutableData;
};

export const InstalledGameModal = (props: Props) => {
  const [modLoaders] = useModLoaders();

  return (
    <Modal
      opened={true}
      onClose={() => props.data.installMod(undefined)}
      title={<GameExecutableName data={props.data} />}
      centered
      size="lg"
    >
      <Stack>
        <Button.Group orientation="vertical">
          <Button
            variant="default"
            onClick={() => startGame(props.data.executable.id)}
          >
            Start Game
          </Button>
          <Button
            variant="default"
            onClick={() => openGameFolder(props.data.executable.id)}
          >
            Open Game Folder
          </Button>
          <Button
            variant="default"
            onClick={() => openGameModsFolder(props.data.executable.id)}
          >
            Open Mods Folder
          </Button>
        </Button.Group>
        {modLoaders.map((modLoader) => (
          <Fragment key={modLoader.id}>
            <label>{modLoader.id} mods</label>
            <Button.Group orientation="vertical">
              {modLoader.mods
                .filter(
                  (mod) =>
                    mod.scriptingBackend ===
                    props.data.executable.scriptingBackend
                )
                .map((mod) => (
                  <Button
                    variant="default"
                    key={mod.name}
                    onClick={() =>
                      installMod(modLoader.id, mod.id, props.data.executable.id)
                    }
                  >
                    Install {mod.name}
                  </Button>
                ))}
            </Button.Group>
          </Fragment>
        ))}
        <label>Debug Data</label>
        <Code sx={{ overflow: "auto" }}>
          <pre>{JSON.stringify(props.data, null, 2)}</pre>
        </Code>
      </Stack>
    </Modal>
  );
};

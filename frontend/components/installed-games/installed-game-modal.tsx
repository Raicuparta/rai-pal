import { Button, Code, Modal, Stack } from "@mantine/core";
import { useModLoaders } from "@hooks/use-backend-data";
import {
  Game,
  installMod,
  openGameFolder,
  openGameModsFolder,
  startGame,
} from "@api/bindings";
import { Fragment } from "react";
import { GameName } from "./game-executable-name";

type Props = {
  readonly game: Game;
  readonly onClose: () => void;
};

export function InstalledGameModal(props: Props) {
  const [modLoaders] = useModLoaders();

  return (
    <Modal
      opened
      onClose={props.onClose}
      title={<GameName game={props.game} />}
      centered
      size="lg"
    >
      <Stack>
        <Button.Group orientation="vertical">
          <Button variant="default" onClick={() => startGame(props.game.id)}>
            Start Game
          </Button>
          <Button
            variant="default"
            onClick={() => openGameFolder(props.game.id)}
          >
            Open Game Folder
          </Button>
          <Button
            variant="default"
            onClick={() => openGameModsFolder(props.game.id)}
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
                  (mod) => mod.scriptingBackend === props.game.scriptingBackend
                )
                .map((mod) => (
                  <Button
                    variant="default"
                    key={mod.name}
                    onClick={() =>
                      installMod(modLoader.id, mod.id, props.game.id)
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
          <pre>{JSON.stringify(props.game, null, 2)}</pre>
        </Code>
      </Stack>
    </Modal>
  );
}
